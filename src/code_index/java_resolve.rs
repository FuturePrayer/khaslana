//! 用已合并的声明解释调用返回值；不读取函数体，也不把未知表达式回退到任意重载。

use super::{ImportContext, Registry};
use super::super::java::{self, Expression, JavaCall};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct Field {
    pub owner: String,
    pub file: Arc<str>,
    pub kind: String,
    pub getter: bool,
}

impl Registry {
    pub(super) fn java_type(&self, kind: &str, file: &str, imports: &ImportContext) -> Option<String> {
        if kind == "java.lang.Class" { return Some("Class".into()); }
        let normalized = java::type_name(kind);
        if normalized.contains(">.") { return None; }
        let kind = java::raw_type(&normalized);
        if let Some(component) = kind.strip_suffix("[]") {
            return self.java_type(component, file, imports).map(|component| format!("{component}[]"));
        }
        if let Some((outer, nested)) = kind.split_once('.') {
            if outer.chars().next().is_some_and(char::is_uppercase) {
                return self.java_type(outer, file, imports).map(|outer| format!("{outer}.{nested}"));
            }
        }
        if kind.contains('.') || matches!(kind, "?" | "null" | "void" | "byte" | "short" | "int" | "long" | "float" | "double" | "char" | "boolean") {
            return Some(kind.into());
        }
        if let Some(types) = imports.java_explicit.get(kind) {
            let first = types.first()?;
            return types.iter().all(|other| other == first).then(|| java::type_name(first));
        }
        if let Some(bucket) = self.name_index.get(kind) {
            let mut local = bucket.candidates.iter().filter(|candidate| candidate.container && candidate.file_path.as_ref() == file);
            if let Some(candidate) = local.next() {
                if local.next().is_some() { return None; }
                return Some(format!("{}.{}", candidate.logical_container.trim_end_matches('.'), kind).trim_start_matches('.').into());
            }
        }
        let package = self.java_packages.get(file).map(String::as_str).unwrap_or("");
        let same_package = if package.is_empty() { kind.into() } else { format!("{package}.{kind}") };
        if self.name_index.get(kind).is_some_and(|bucket| bucket.candidates.iter().any(|candidate| candidate.container
            && format!("{}.{}", candidate.logical_container.trim_end_matches('.'), kind).trim_start_matches('.') == same_package)) {
            return Some(same_package);
        }
        if matches!(kind, "String" | "Object" | "Number" | "Integer" | "Long" | "Double" | "Float" | "Short" | "Byte" | "Character" | "Boolean" | "CharSequence" | "Comparable" | "Iterable" | "Cloneable" | "Class") {
            return Some(kind.into());
        }
        let wildcard: Vec<_> = imports.java_wildcards.iter().filter_map(|package| {
            let name = format!("{package}.{kind}");
            (self.name_index.get(kind).is_some_and(|bucket| bucket.candidates.iter().any(|candidate| candidate.container
                && format!("{}.{}", candidate.logical_container.trim_end_matches('.'), kind) == name))
                || package == "java.util" && matches!(kind, "List" | "Map" | "Collection" | "Set" | "ArrayList" | "LinkedList" | "HashMap" | "LinkedHashMap" | "HashSet"))
                .then_some(name)
        }).collect();
        match wildcard.as_slice() { [name] => Some(name.clone()), [] => Some(same_package), _ => None }
    }

    pub(super) fn java_call(&self, call: &JavaCall, file: &str, imports: &ImportContext, scope: &[String], depth: usize) -> JavaCall {
        let mut typed = call.clone();
        for (index, argument) in typed.arguments.iter_mut().enumerate() {
            *argument = argument.as_deref().and_then(|kind| self.java_type(kind, file, imports)).or_else(|| {
                call.argument_expressions.get(index).and_then(Option::as_ref)
                    .and_then(|expression| self.java_value(expression, file, imports, scope, depth + 1))
            });
        }
        typed.receiver_type = call.receiver_type.as_deref().and_then(|kind| self.java_type(kind, file, imports)).or_else(|| {
            call.receiver_expression.as_deref().and_then(|expression| self.java_value(expression, file, imports, scope, depth + 1))
        });
        // 表达式存在但仍不能推导时，禁止把链式接收者当类名或全仓库唯一方法。
        if call.receiver_expression.is_some() && typed.receiver_type.is_none() { typed.unknown_receiver = true; }
        typed
    }

    fn java_value(&self, expression: &Expression, file: &str, imports: &ImportContext, scope: &[String], depth: usize) -> Option<String> {
        if depth > 12 { return None; }
        match expression {
            Expression::Type(kind) => self.java_type(kind, file, imports),
            Expression::This => {
                let package = self.java_packages.get(file).map(String::as_str).unwrap_or("");
                (!scope.is_empty()).then(|| format!("{package}.{}", scope.join(".")).trim_start_matches('.').into())
            }
            Expression::Field { receiver, name } => {
                let owner = self.java_value(receiver, file, imports, scope, depth + 1)?;
                self.java_field(&owner, name, false)
            }
            Expression::Call { name, qualifier, call: original } => {
                let call = self.java_call(original, file, imports, scope, depth + 1);
                if call.unknown_receiver { return None; }
                if let Some(target) = self.resolve_typed_inner(name, file, imports, qualifier.as_deref(), scope, Some(&call)) {
                    let candidate = self.name_index.get(name)?.candidates.iter().find(|candidate| candidate.id == target.id)?;
                    let kind = candidate.java_definition.value_type.as_deref()?;
                    if candidate.java_definition.type_parameters.iter().any(|parameter| parameter == kind) || kind == "void" { return None; }
                    let empty = ImportContext::default();
                    return self.java_type(kind, &candidate.file_path, self.java_imports.get(&candidate.file_path).unwrap_or(&empty));
                }
                if let Some(kind) = self.java_collection_get(original, &call, name, file, imports) { return Some(kind); }
                if !call.arguments.is_empty() { return None; }
                let owner = call.receiver_type?;
                // 显式方法优先；重载歧义不能借自动 getter 的名字绕过。
                if self.name_index.get(name).is_some_and(|bucket| bucket.candidates.iter().any(|candidate| candidate.callable && candidate.logical_container == owner)) { return None; }
                let field = name.strip_prefix("get").or_else(|| name.strip_prefix("is"))?;
                let mut chars = field.chars();
                let first = chars.next()?;
                let field = if field.chars().nth(1).is_some_and(char::is_uppercase) { field.into() }
                    else { first.to_lowercase().chain(chars).collect::<String>() };
                let kind = self.java_field(&owner, &field, true)?;
                if name.starts_with("is") && kind != "boolean" { return None; }
                Some(kind)
            }
        }
    }

    fn java_field(&self, owner: &str, name: &str, getter: bool) -> Option<String> {
        let mut fields = self.java_fields.get(name)?.iter().filter(|field| field.owner == owner && (!getter || field.getter));
        let field = fields.next()?;
        if fields.next().is_some() { return None; }
        let empty = ImportContext::default();
        self.java_type(&field.kind, &field.file, self.java_imports.get(&field.file).unwrap_or(&empty))
    }

    fn java_collection_get(&self, original: &JavaCall, typed: &JavaCall, name: &str, file: &str, imports: &ImportContext) -> Option<String> {
        if name != "get" || typed.arguments.len() != 1 { return None; }
        let index = match typed.receiver_type.as_deref()? {
            "java.util.Map" | "java.util.HashMap" | "java.util.LinkedHashMap" => 1,
            "java.util.List" | "java.util.ArrayList" | "java.util.LinkedList" => 0,
            _ => return None,
        };
        // 仅从原始声明读取泛型；链式返回已擦除时不能把未知值类型猜成 Object。
        let receiver = original.receiver_type.as_deref()?;
        let Some((_, arguments)) = receiver.split_once('<') else { return Some("Object".into()); };
        let arguments = arguments.strip_suffix('>')?;
        let mut depth = 0usize;
        let mut start = 0usize;
        let mut types = Vec::new();
        for (offset, ch) in arguments.char_indices() {
            match ch {
                '<' => depth += 1,
                '>' => depth = depth.checked_sub(1)?,
                ',' if depth == 0 => { types.push(arguments[start..offset].trim()); start = offset + 1; }
                _ => {}
            }
        }
        types.push(arguments[start..].trim());
        let kind = *types.get(index)?;
        if kind.contains('?') || kind.len() == 1 && kind.chars().all(|ch| ch.is_ascii_uppercase()) { return None; }
        self.java_type(kind, file, imports)
    }
}
