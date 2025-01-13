use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::path::Path;
use swc_common::errors::Handler;
use swc_common::sync::Lrc;
use swc_common::{errors::ColorConfig, SourceMap};
use swc_ecma_ast::{CallExpr, Callee, ExprOrSpread};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};

/**
 * 0:Not a string literal
 * 1: Same key with different value
 * 2: Same key with different value exists in output file
 * 3: args is empty
 */
// const err_type_tup: (u8, u8, u8, u8) = (0, 1, 2, 3);

pub struct IntlInfo {
    pub info_map: HashMap<String, IntlOkInfo>,
    pub repeat_key_list: Vec<(String, Value)>,
    pub err_map: HashMap<u8, Vec<IntlErrInfo>>,
}

impl Display for IntlInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let add_num = self.info_map.len();
        let mut err_str = "".to_string();
        if self.err_map.contains_key(&0) {
            err_str.push_str("\n ============== Not a string literal =============\n");
            self.err_map.get(&0).unwrap().iter().for_each(|e| {
                err_str.push_str(format!("\n {};", e.err_msg).as_str());
            });
        }
        if self.err_map.contains_key(&1) {
            err_str.push_str("\n ============== Same key with different value =============\n");
            self.err_map.get(&1).unwrap().iter().for_each(|e| {
                err_str.push_str(format!("\n {};", e.err_msg).as_str());
            });
        }
        if self.err_map.contains_key(&2) {
            err_str.push_str(
                "\n ============== Same key with different value from existed file =============\n",
            );
            self.err_map.get(&2).unwrap().iter().for_each(|e| {
                err_str.push_str(format!("\n {};", e.err_msg).as_str());
            });
        }
        if self.err_map.contains_key(&3) {
            err_str.push_str("\n ============== Args is empty  =============\n");
            self.err_map.get(&3).unwrap().iter().for_each(|e| {
                err_str.push_str(format!("\n {};", e.err_msg).as_str());
            });
        }
        return write!(
            f,
            "\n ************** Complete the extraction of {} pieces of text. **************\n{}",
            add_num, err_str
        );
    }
}

pub struct IntlErrInfo {
    pub err_type: u8,
    pub err_msg: String,
}

impl IntlErrInfo {
    fn new(err_type: u8, err_msg: String) -> IntlErrInfo {
        IntlErrInfo { err_type, err_msg }
    }
}

#[derive(Debug)]
pub struct IntlOkInfo {
    pub key: String,
    pub default: String,
}

// #[derive(Default)]
struct TransformVisitor<'a> {
    // already exist
    existed_map: &'a Map<String, Value>,
    // visited
    visited_intl: &'a mut IntlInfo,
    // current file path
    cm: &'a SourceMap,
}

// 输出格式:
// 全部XXX条,新增XXX条
// 不是常量/不是字符串/参数为空

fn get_caller_name(callee: &Callee) -> Option<&str> {
    if callee.is_expr() {
        let name = callee.as_expr()?.as_ident()?.sym.as_str();
        return Some(name);
    }
    return None;
}

struct ErrInfo {
    err_type: u8,
    err_msg: String,
}

fn get_intel_info(node: &CallExpr) -> Result<&str, ErrInfo> {
    let args: &Vec<ExprOrSpread> = &node.args;
    if args.len() >= 1 {
        let args_0 = &args[0];
        let option_lit = args_0.expr.as_lit();
        if let Some(lit) = option_lit {
            let option_str = lit.as_str();
            if let Some(str) = option_str {
                return Ok(&str.value);
            } else {
                return Err(ErrInfo {
                    err_type: 0,
                    err_msg: "Error: Not a string literal.".to_string(),
                });
            }
        } else {
            return Err(ErrInfo {
                err_type: 0,
                err_msg: "Error: Not a string literal.".to_string(),
            });
        }
    } else {
        return Err(ErrInfo {
            err_type: 3,
            err_msg: "Error: Args is empty.".to_string(),
        });
    }
}

impl Visit for TransformVisitor<'_> {
    fn visit_call_expr(&mut self, node: &CallExpr) {
        let callee = &node.callee;
        let name = get_caller_name(callee);
        // let curr_path = &self.curr_path;
        let detail_msg = format!(
            " File is {file}; {line}",
            file = self
                .cm
                .span_to_lines(node.span)
                .unwrap()
                .file
                .name
                .to_string(),
            line = self
                .cm
                .span_to_lines(node.span)
                .unwrap()
                .lines
                .iter()
                .map(|l| format!(
                    "Line num: {index}, start col num: {start}, end col num: {end} ",
                    index = l.line_index,
                    start = l.start_col.0,
                    end = l.end_col.0
                ))
                .collect::<Vec<_>>()
                .join(";")
        );

        if let Some(n) = name {
            if n == "$t" {
                let value = get_intel_info(node);
                match value {
                    Ok(v) => {
                        // unique id
                        let id = v;
                        // default value
                        let default_val = v;
                        if self.visited_intl.info_map.contains_key(id) {
                            let visited_intl = self.visited_intl.info_map.get(id).unwrap();
                            if visited_intl.default != default_val {
                                let err_msg = "Error: Intl key:".to_string()
                                    + id
                                    + " same key with different value"
                                    + detail_msg.as_str();
                                let error_info = IntlErrInfo::new(1, err_msg);
                                if self.visited_intl.err_map.contains_key(&1) {
                                    self.visited_intl
                                        .err_map
                                        .get_mut(&1)
                                        .unwrap()
                                        .push(error_info);
                                } else {
                                    self.visited_intl.err_map.insert(1, vec![error_info]);
                                }
                            }
                        } else if self.existed_map.contains_key(id) {
                            let existed_value = self.existed_map.get(id).unwrap();
                            self.visited_intl
                                .repeat_key_list
                                .push((id.to_string(), existed_value.clone()));

                            if existed_value != default_val {
                                let err_msg = "Error: Intl key: ".to_string()
                                    + id
                                    + " same key with different value from existed file'"
                                    + detail_msg.as_str();
                                let error_info = IntlErrInfo::new(2, err_msg);
                                if self.visited_intl.err_map.contains_key(&2) {
                                    self.visited_intl
                                        .err_map
                                        .get_mut(&2)
                                        .unwrap()
                                        .push(error_info);
                                } else {
                                    self.visited_intl.err_map.insert(2, vec![error_info]);
                                }
                            }
                        } else {
                            self.visited_intl.info_map.insert(
                                id.to_string(),
                                IntlOkInfo {
                                    key: id.to_string(),
                                    default: default_val.to_string(),
                                },
                            );
                        }
                    }
                    Err(e) => {
                        let error_info =
                            IntlErrInfo::new(e.err_type, e.err_msg + detail_msg.as_str());
                        if self.visited_intl.err_map.contains_key(&e.err_type) {
                            self.visited_intl
                                .err_map
                                .get_mut(&e.err_type)
                                .unwrap()
                                .push(error_info);
                        } else {
                            self.visited_intl
                                .err_map
                                .insert(e.err_type, vec![error_info]);
                        }
                    }
                }
            }
        }
        node.visit_children_with(self);
    }
}

pub fn extract_text<'a>(
    path: &str,
    existed_map: &'a Map<String, Value>,
    extract_info: &mut IntlInfo,
) {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let fm = cm
        .load_file(Path::new(path))
        .expect(&(String::from("Failed to load ") + path));
    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            dts: false,
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        }),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }
    let _module = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect(&(String::from("Failed to parser module") + path));

    let mut visitor: TransformVisitor<'_> = TransformVisitor {
        existed_map: existed_map,
        visited_intl: extract_info,
        cm: &cm,
    };
    _module.visit_with(&mut visitor);
}
