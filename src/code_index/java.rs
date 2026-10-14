//! Java 的局部类型证据。只使用声明和字面量，不把无法推导的表达式猜成类型名。

use std::collections::HashMap;
use tree_sitter::Node;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct JavaCall {
    pub arguments: Vec<Option<String>>,
    pub receiver_type: Option<String>,
    pub instance: bool,
    pub unknown_receiver: bool,
}

#[derive(Default)]
pub(super) struct Context {
    frames: Vec<HashMap<String, Option<String>>>,
    classes: Vec<usize>,
}

fn text<'a>(node: Node, source: &'a [u8]) -> &'a str { node.utf8_text(source).unwrap_or_default() }

pub(super) fn type_name(value: &str) -> String {
    let value = value.trim();
    match value {
        "java.lang.String" => "String".into(),
        "java.lang.Object" => "Object".into(),
        _ => value.to_string(),
    }
}

pub(super) fn parameters(node: Node, source: &[u8]) -> Option<Vec<String>> {
    let parameters = node.child_by_field_name("parameters")?;
    let mut cursor = parameters.walk();
    Some(parameters.named_children(&mut cursor).filter(|child| matches!(child.kind(), "formal_parameter" | "spread_parameter"))
        .map(|parameter| {
            // 变参与后置数组维度保留未知；泛型不擦除，避免制造唯一候选。
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

    pub fn call(&self, node: Node, source: &[u8]) -> JavaCall {
        let mut arguments = Vec::new();
        if let Some(list) = node.child_by_field_name("arguments") {
            let mut cursor = list.walk();
            arguments.extend(list.named_children(&mut cursor).filter(|child| !matches!(child.kind(), "line_comment" | "block_comment"))
                .map(|argument| self.expression(argument, source)));
        }
        let receiver = node.child_by_field_name("object");
        let receiver_type = receiver.and_then(|receiver| self.expression(receiver, source));
        let instance = receiver_type.is_some() || receiver.is_some_and(|receiver| receiver.kind() == "this");
        let unknown_receiver = receiver.is_some_and(|receiver| {
            let qualified_class = receiver.kind() == "field_access" && {
                let first = text(receiver, source).split('.').next().unwrap_or_default();
                !matches!(first, "this" | "super") && self.lookup(first).is_none()
            };
            !qualified_class && !matches!(receiver.kind(), "this" | "identifier" | "scoped_identifier")
                || receiver.kind() == "identifier" && self.lookup(text(receiver, source)).is_some() && receiver_type.is_none()
        }) && !instance;
        JavaCall { arguments, receiver_type, instance, unknown_receiver }
    }
}

fn primitive(value: &str) -> bool { matches!(value, "byte" | "short" | "int" | "long" | "float" | "double" | "char" | "boolean") }

// 只比较已知的安全特例；未知继承、泛型、装箱关系不排除候选。
fn conversion(argument: &str, parameter: &str) -> Option<u8> {
    if argument == parameter { return Some(0); }
    if parameter == "Object" && primitive(argument) { return Some(3); }
    if parameter == "Object" && !primitive(argument) && argument != "?" { return Some(2); }
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
                !(argument == "null" && primitive(parameter)
                    || primitive(argument) && parameter == "String"
                    || argument == "String" && primitive(parameter)
                    || argument == "Object" && (parameter == "String" || primitive(parameter))
                    || primitive(argument) && primitive(parameter) && conversion(argument, parameter).is_none())
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
                if argument == "null" && right == "Object" && left != "Object" { better = true; }
            }
            better
        })
    })
}
