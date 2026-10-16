//! Java 的局部类型证据。保留有界表达式，在定义全部合并后推导返回类型。

use std::collections::HashMap;
use tree_sitter::Node;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct JavaCall {
    pub arguments: Vec<Option<String>>,
    pub receiver_type: Option<String>,
    pub instance: bool,
    pub unknown_receiver: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub argument_expressions: Vec<Option<Expression>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver_expression: Option<Box<Expression>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Expression {
    Type(String),
    This,
    Call { name: String, qualifier: Option<String>, call: Box<JavaCall> },
    Field { receiver: Box<Expression>, name: String },
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub value_type: Option<String>,
    pub getter: bool,
    pub type_parameters: Vec<String>,
}

pub(super) fn definition(node: Node, source: &[u8]) -> Definition {
    let mut value_type = node.child_by_field_name("type").map(|kind| type_name(text(kind, source)));
    if node.kind() == "field_declaration" {
        let mut cursor = node.walk();
        if node.named_children(&mut cursor).filter(|child| child.kind() == "variable_declarator")
            .any(|variable| variable.child_by_field_name("dimensions").is_some()) { value_type = None; }
    }
    let mut type_parameters = Vec::new();
    let mut ancestor = Some(node);
    let mut getter = false;
    while let Some(current) = ancestor {
        if let Some(parameters) = current.child_by_field_name("type_parameters") {
            let mut cursor = parameters.walk();
            for parameter in parameters.named_children(&mut cursor) {
                if let Some(name) = parameter.named_child(0) { type_parameters.push(text(name, source).to_string()); }
            }
        }
        if node.kind() == "field_declaration" && matches!(current.kind(), "field_declaration" | "class_declaration") {
            let mut cursor = current.walk();
            if let Some(modifiers) = current.named_children(&mut cursor).find(|child| child.kind() == "modifiers") {
                let annotations = text(modifiers, source);
                // 仅凭字段名不能假设 getter 存在；只接受来源可核对的 Lombok 声明。
                if !annotations.contains("AccessLevel.NONE") {
                    let mut annotations_cursor = modifiers.walk();
                    getter |= modifiers.named_children(&mut annotations_cursor).filter_map(|annotation| annotation.child_by_field_name("name"))
                        .any(|annotation| ["Data", "Getter", "Value"].iter().any(|name| {
                            let annotation = text(annotation, source);
                            annotation == format!("lombok.{name}") || annotation == *name && lombok_imported(node, source, name)
                        }));
                } else { return Definition { value_type, getter: false, type_parameters }; }
            }
        }
        ancestor = current.parent();
    }
    Definition { value_type, getter, type_parameters }
}

fn lombok_imported(mut node: Node, source: &[u8], name: &str) -> bool {
    while let Some(parent) = node.parent() { node = parent; }
    let mut cursor = node.walk();
    node.named_children(&mut cursor).filter(|child| child.kind() == "import_declaration").any(|import| {
        let import = text(import, source).split_whitespace().collect::<String>();
        import == format!("importlombok.{name};") || import == "importlombok.*;"
    })
}

#[derive(Default)]
pub(super) struct Context {
    frames: Vec<HashMap<String, Option<String>>>,
    classes: Vec<usize>,
}

fn text<'a>(node: Node, source: &'a [u8]) -> &'a str { node.utf8_text(source).unwrap_or_default() }

pub(super) fn type_name(value: &str) -> String {
    let normalized = value.split_whitespace().collect::<String>();
    let value = normalized.as_str();
    match value {
        "java.lang.String" => "String".into(),
        "java.lang.Object" => "Object".into(),
        "java.lang.Number" => "Number".into(),
        "java.lang.Integer" => "Integer".into(),
        "java.lang.Long" => "Long".into(),
        "java.lang.Double" => "Double".into(),
        "java.lang.Float" => "Float".into(),
        "java.lang.Short" => "Short".into(),
        "java.lang.Byte" => "Byte".into(),
        "java.lang.Character" => "Character".into(),
        "java.lang.Boolean" => "Boolean".into(),
        "java.lang.CharSequence" => "CharSequence".into(),
        "java.lang.Comparable" => "Comparable".into(),
        "java.lang.Iterable" => "Iterable".into(),
        "java.lang.Cloneable" => "Cloneable".into(),
        _ => value.to_string(),
    }
}

pub(super) fn raw_type(value: &str) -> &str {
    value.split_once('<').map_or(value, |(name, _)| name)
}

pub(super) fn parameters(node: Node, source: &[u8]) -> Option<Vec<String>> {
    let parameters = node.child_by_field_name("parameters")?;
    let mut cursor = parameters.walk();
    Some(parameters.named_children(&mut cursor).filter(|child| matches!(child.kind(), "formal_parameter" | "spread_parameter"))
        .map(|parameter| {
            // 变参与后置数组维度保留未知；泛型先保留，解析阶段按导入身份比较原始类型。
            if parameter.kind() == "spread_parameter" || parameter.child_by_field_name("dimensions").is_some() { return "?".into(); }
            parameter.child_by_field_name("type").map(|kind| type_name(text(kind, source))).unwrap_or_else(|| "?".into())
        }).collect())
}

impl Context {
    pub fn enter(&mut self, node: Node, source: &[u8]) -> bool {
        let frame = matches!(node.kind(), "class_body" | "interface_body" | "enum_body" | "method_declaration"
            | "constructor_declaration" | "block" | "for_statement" | "enhanced_for_statement" | "catch_clause" | "lambda_expression");
        if frame {
            self.frames.push(HashMap::new());
            if matches!(node.kind(), "class_body" | "interface_body" | "enum_body") {
                self.classes.push(self.frames.len() - 1);
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor).filter(|child| child.kind() == "field_declaration") {
                    self.declaration(child, source);
                }
            }
        }
        if matches!(node.kind(), "formal_parameter" | "catch_formal_parameter" | "enhanced_for_statement") {
            self.bind(node, source);
        }
        if node.kind() == "spread_parameter" {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor).filter(|child| child.kind() == "variable_declarator") {
                if let Some(name) = child.child_by_field_name("name") { self.insert(text(name, source), None); }
            }
        }
        if node.kind() == "variable_declarator" && let Some(parent) = node.parent()
            && parent.kind() == "local_variable_declaration" {
            let kind = parent.child_by_field_name("type").map(|kind| type_name(text(kind, source)));
            let kind = if node.child_by_field_name("dimensions").is_some() { None }
                else if kind.as_deref() == Some("var") { node.child_by_field_name("value").and_then(|value| self.expression(value, source)) } else { kind };
            if let Some(name) = node.child_by_field_name("name") { self.insert(text(name, source), kind); }
        }
        // 未声明类型的 lambda 参数仍必须遮蔽外层同名变量。
        if node.kind() == "lambda_expression" && let Some(parameters) = node.child_by_field_name("parameters") {
            if parameters.kind() == "identifier" { self.insert(text(parameters, source), None); }
            if parameters.kind() == "inferred_parameters" {
                let mut cursor = parameters.walk();
                for name in parameters.named_children(&mut cursor) { self.insert(text(name, source), None); }
            }
        }
        frame
    }

    pub fn leave(&mut self) {
        if self.classes.last().copied() == Some(self.frames.len() - 1) { self.classes.pop(); }
        self.frames.pop();
    }

    fn insert(&mut self, name: &str, kind: Option<String>) {
        if let Some(frame) = self.frames.last_mut() { frame.insert(name.into(), kind); }
    }

    fn bind(&mut self, node: Node, source: &[u8]) {
        if let Some(name) = node.child_by_field_name("name") {
            let kind = if node.child_by_field_name("dimensions").is_some() { None }
                else { node.child_by_field_name("type").map(|kind| type_name(text(kind, source))) };
            self.insert(text(name, source), kind);
        }
    }

    fn declaration(&mut self, node: Node, source: &[u8]) {
        let kind = node.child_by_field_name("type").map(|kind| type_name(text(kind, source)));
        let mut cursor = node.walk();
        for variable in node.named_children(&mut cursor).filter(|child| child.kind() == "variable_declarator") {
            if let Some(name) = variable.child_by_field_name("name") {
                let kind = if variable.child_by_field_name("dimensions").is_some() { None } else { kind.clone() };
                self.insert(text(name, source), kind);
            }
        }
    }

    fn lookup(&self, name: &str) -> Option<&Option<String>> {
        self.frames.iter().rev().find_map(|frame| frame.get(name))
    }

    pub fn expression(&self, node: Node, source: &[u8]) -> Option<String> {
        let primitive = match node.kind() {
            "class_literal" => Some("java.lang.Class"),
            "string_literal" => Some("String"), "character_literal" => Some("char"),
            "true" | "false" => Some("boolean"), "null_literal" => Some("null"),
            "decimal_integer_literal" | "hex_integer_literal" | "octal_integer_literal" | "binary_integer_literal" =>
                Some(if text(node, source).ends_with(['l', 'L']) { "long" } else { "int" }),
            "decimal_floating_point_literal" | "hex_floating_point_literal" =>
                Some(if text(node, source).ends_with(['f', 'F']) { "float" } else { "double" }),
            _ => None,
        };
        if let Some(kind) = primitive { return Some(kind.into()); }
        match node.kind() {
            "identifier" => self.lookup(text(node, source)).cloned().flatten(),
            "object_creation_expression" | "cast_expression" => node.child_by_field_name("type").map(|kind| type_name(text(kind, source))),
            "parenthesized_expression" => node.named_child(0).and_then(|child| self.expression(child, source)),
            "field_access" if node.child_by_field_name("object").is_some_and(|object| object.kind() == "this") => {
                let name = node.child_by_field_name("field")?;
                self.classes.last().and_then(|index| self.frames[*index].get(text(name, source))).cloned().flatten()
            }
            _ => None,
        }
    }

    fn value(&self, node: Node, source: &[u8], depth: usize) -> Option<Expression> {
        if depth > 12 { return None; }
        if let Some(kind) = self.expression(node, source) { return Some(Expression::Type(kind)); }
        match node.kind() {
            "this" => Some(Expression::This),
            "method_invocation" => Some(Expression::Call {
                name: text(node.child_by_field_name("name")?, source).into(),
                qualifier: node.child_by_field_name("object").map(|object| text(object, source).into()),
                call: Box::new(self.call_at(node, source, depth + 1)),
            }),
            "field_access" => Some(Expression::Field {
                receiver: Box::new(self.value(node.child_by_field_name("object")?, source, depth + 1)?),
                name: text(node.child_by_field_name("field")?, source).into(),
            }),
            "parenthesized_expression" => self.value(node.named_child(0)?, source, depth + 1),
            _ => None,
        }
    }

    pub fn call(&self, node: Node, source: &[u8]) -> JavaCall { self.call_at(node, source, 0) }

    fn call_at(&self, node: Node, source: &[u8], depth: usize) -> JavaCall {
        let mut arguments = Vec::new();
        let mut argument_expressions = Vec::new();
        if let Some(list) = node.child_by_field_name("arguments") {
            let mut cursor = list.walk();
            for argument in list.named_children(&mut cursor).filter(|child| !matches!(child.kind(), "line_comment" | "block_comment")) {
                let kind = self.expression(argument, source);
                argument_expressions.push(if kind.is_none() { self.value(argument, source, depth + 1) } else { None });
                arguments.push(kind);
            }
        }
        let receiver = node.child_by_field_name("object");
        let receiver_type = receiver.and_then(|receiver| self.expression(receiver, source));
        let receiver_expression = receiver.filter(|_| receiver_type.is_none()).and_then(|receiver| self.value(receiver, source, depth + 1)).map(Box::new);
        let instance = receiver_type.is_some() || receiver_expression.is_some() || receiver.is_some_and(|receiver| receiver.kind() == "this");
        let unknown_receiver = receiver.is_some_and(|receiver| {
            let qualified_class = receiver.kind() == "field_access" && {
                let first = text(receiver, source).split('.').next().unwrap_or_default();
                !matches!(first, "this" | "super") && self.lookup(first).is_none()
            };
            !qualified_class && !matches!(receiver.kind(), "this" | "identifier" | "scoped_identifier")
                || receiver.kind() == "identifier" && self.lookup(text(receiver, source)).is_some() && receiver_type.is_none()
        }) && !instance;
        if argument_expressions.iter().all(Option::is_none) { argument_expressions.clear(); }
        JavaCall { arguments, receiver_type, instance, unknown_receiver, argument_expressions, receiver_expression }
    }
}

impl JavaCall {
    pub(super) fn field_dependencies(&self, names: &mut std::collections::HashSet<String>) {
        fn visit(expression: &Expression, names: &mut std::collections::HashSet<String>) {
            match expression {
                Expression::Field { receiver, name } => { names.insert(name.clone()); visit(receiver, names); }
                Expression::Call { call, .. } => call.field_dependencies(names),
                Expression::Type(_) | Expression::This => {}
            }
        }
        for expression in self.argument_expressions.iter().flatten().chain(self.receiver_expression.as_deref()) {
            visit(expression, names);
        }
    }
}

fn primitive(value: &str) -> bool { matches!(value, "byte" | "short" | "int" | "long" | "float" | "double" | "char" | "boolean") }

fn unboxed(value: &str) -> Option<&'static str> {
    match value {
        "Byte" => Some("byte"), "Short" => Some("short"), "Integer" => Some("int"), "Long" => Some("long"),
        "Float" => Some("float"), "Double" => Some("double"), "Character" => Some("char"), "Boolean" => Some("boolean"), _ => None,
    }
}

fn collection_family(value: &str) -> Option<u8> {
    match value {
        "java.util.List" | "java.util.ArrayList" | "java.util.LinkedList" => Some(1),
        "java.util.Map" | "java.util.HashMap" | "java.util.LinkedHashMap" => Some(2),
        "java.util.Set" | "java.util.HashSet" => Some(3), _ => None,
    }
}

fn reference_parents(value: &str) -> Option<&'static [&'static str]> {
    Some(match value {
        "String" => &["Object", "CharSequence", "Comparable", "java.io.Serializable"],
        "Integer" | "Long" | "Double" | "Float" | "Short" | "Byte" | "java.math.BigDecimal" | "java.math.BigInteger" => &["Number", "Object", "Comparable", "java.io.Serializable"],
        "Boolean" | "Character" => &["Object", "Comparable", "java.io.Serializable"],
        "Number" => &["Object", "java.io.Serializable"],
        "java.util.ArrayList" => &["java.util.List", "java.util.Collection", "Iterable", "Object", "java.io.Serializable", "Cloneable", "java.util.RandomAccess"],
        "java.util.LinkedList" => &["java.util.List", "java.util.Deque", "java.util.Queue", "java.util.Collection", "Iterable", "Object", "java.io.Serializable", "Cloneable"],
        "java.util.List" | "java.util.Set" => &["java.util.Collection", "Iterable", "Object"],
        "java.util.Collection" => &["Iterable", "Object"],
        "java.util.HashMap" => &["java.util.Map", "Object", "java.io.Serializable", "Cloneable"],
        "java.util.LinkedHashMap" => &["java.util.HashMap", "java.util.Map", "Object", "java.io.Serializable", "Cloneable"],
        "java.util.HashSet" => &["java.util.Set", "java.util.Collection", "Iterable", "Object", "java.io.Serializable", "Cloneable"],
        "java.util.Map" | "CharSequence" | "Comparable" | "Iterable" | "java.io.Serializable" | "Cloneable" => &["Object"],
        _ => return None,
    })
}

fn incompatible(argument: &str, parameter: &str) -> bool {
    if argument == "?" || parameter == "?" { return false; }
    if argument == "null" { return primitive(parameter); }
    if let Some(parameter) = parameter.strip_suffix("[]") {
        let Some(argument) = argument.strip_suffix("[]") else { return true; };
        return if primitive(argument) || primitive(parameter) { argument != parameter } else { incompatible(argument, parameter) };
    }
    if argument.ends_with("[]") { return !matches!(parameter, "Object" | "Cloneable" | "java.io.Serializable"); }
    if parameter == "String" { return argument != "String"; }
    if parameter == "Class" { return argument != "Class"; }
    if argument == "Object" { return parameter != "Object"; }
    if primitive(parameter) && !primitive(argument) {
        return unboxed(argument).is_none_or(|kind| conversion(kind, parameter).is_none());
    }
    if primitive(argument) && unboxed(parameter).is_some() { return unboxed(parameter) != Some(argument); }
    if primitive(argument) && parameter == "Number" { return matches!(argument, "char" | "boolean"); }
    if unboxed(argument).is_some() && unboxed(parameter).is_some() { return argument != parameter; }
    if primitive(argument) && primitive(parameter) { return conversion(argument, parameter).is_none(); }
    if !primitive(parameter) && let Some(parents) = reference_parents(argument) {
        return argument != parameter && !parents.contains(&parameter);
    }
    if collection_family(argument).is_some() && collection_family(parameter).is_some() {
        return conversion(argument, parameter).is_none();
    }
    false
}

// 只比较已知的转换；未知继承关系不参与候选之间的优先级推断。
fn conversion(argument: &str, parameter: &str) -> Option<u8> {
    if argument == parameter { return Some(0); }
    if let (Some(argument), Some(parameter)) = (argument.strip_suffix("[]"), parameter.strip_suffix("[]")) {
        return (!incompatible(argument, parameter)).then_some(1);
    }
    if parameter == "Object" && primitive(argument) { return Some(3); }
    if parameter == "Object" && !primitive(argument) && argument != "?" { return Some(2); }
    if parameter == "Number" && (unboxed(argument).is_some_and(|kind| !matches!(kind, "char" | "boolean"))
        || primitive(argument) && !matches!(argument, "char" | "boolean")) { return Some(2); }
    if unboxed(parameter) == Some(argument) || unboxed(argument) == Some(parameter) { return Some(3); }
    if primitive(parameter) && let Some(kind) = unboxed(argument) { return conversion(kind, parameter).map(|rank| rank + 3); }
    if reference_parents(argument).is_some_and(|parents| parents.contains(&parameter)) { return Some(1); }
    if matches!(argument, "java.util.ArrayList" | "java.util.LinkedList") && parameter == "java.util.List"
        || matches!(argument, "java.util.HashMap" | "java.util.LinkedHashMap") && parameter == "java.util.Map"
        || argument == "java.util.HashSet" && parameter == "java.util.Set"
        || matches!(collection_family(argument), Some(1 | 3)) && parameter == "java.util.Collection" { return Some(1); }
    if argument == "null" && !primitive(parameter) && !parameter.contains(['<', '[']) && parameter != "?" { return Some(1); }
    if primitive(argument) && primitive(parameter) {
        let allowed: &[&str] = match argument {
            "byte" => &["short", "int", "long", "float", "double"], "short" | "char" => &["int", "long", "float", "double"],
            "int" => &["long", "float", "double"], "long" => &["float", "double"], "float" => &["double"], _ => &[],
        };
        return allowed.iter().position(|kind| *kind == parameter).map(|index| index as u8 + 1);
    }
    None
}

pub(super) fn select<'a, T>(candidates: Vec<&'a T>, parameters: impl Fn(&T) -> Option<&Vec<String>>, call: Option<&JavaCall>) -> Option<&'a T> {
    let Some(call) = call else { return (candidates.len() == 1).then(|| candidates[0]); };
    let candidates: Vec<_> = candidates.into_iter().filter(|candidate| parameters(candidate).is_none_or(|parameters| {
        if parameters.iter().any(|parameter| parameter == "?") { return true; }
        parameters.len() == call.arguments.len() && parameters.iter().zip(&call.arguments).all(|(parameter, argument)| {
            argument.as_deref().is_none_or(|argument| {
                !incompatible(argument, parameter)
            })
        })
    })).collect();
    if candidates.len() == 1 { return Some(candidates[0]); }
    // 只有一个候选在所有实参上确定优于其他候选才建边；未知类型不能让候选消失。
    candidates.iter().copied().find(|candidate| {
        let Some(left) = parameters(candidate) else { return false; };
        if left.len() != call.arguments.len() { return false; }
        candidates.iter().filter(|other| !std::ptr::eq(**other, *candidate)).all(|other| {
            let Some(right) = parameters(other) else { return false; };
            if right.len() != left.len() { return false; }
            let mut better = false;
            for ((argument, left), right) in call.arguments.iter().zip(left).zip(right) {
                if left == right { continue; }
                let Some(argument) = argument.as_deref() else { return false; };
                let (Some(a), Some(b)) = (conversion(argument, left), conversion(argument, right)) else { return false; };
                if a > b { return false; }
                if a < b { better = true; }
                if a == b && reference_parents(left).is_some_and(|parents| parents.contains(&right.as_str())) { better = true; }
                if argument == "null" && right == "Object" && left != "Object" { better = true; }
            }
            better
        })
    })
}
