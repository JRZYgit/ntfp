use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Let,
    Print,
    Method,
    Fun,
    Back,
    Identifier,
    Number,
    String,
    Plus,
    Minus,
    Assign,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Multiply,
    Divide,
    Mismatch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub type_: TokenType,
    pub value: String,
    pub line: u32,
    pub position: usize,
}

pub fn lexer(code: &str) -> Result<Vec<Token>, String> {
    let token_specs = [
        (TokenType::Let, r"let"),
        (TokenType::Print, r"print"),
        (TokenType::Method, r"method"),
        (TokenType::Fun, r"fun"),
        (TokenType::Back, r"back"),
        (TokenType::Identifier, r"[a-zA-Z_][a-zA-Z0-9_]*"),
        (TokenType::Number, r"\d+"),
        (TokenType::String, r#""[^"]*""#),
        (TokenType::Plus, r"\+"),
        (TokenType::Minus, r"-"),
        (TokenType::Assign, r"="),
        (TokenType::Semicolon, r";"),
        (TokenType::LParen, r"\("),
        (TokenType::RParen, r"\)"),
        (TokenType::LBrace, r"\{"),
        (TokenType::RBrace, r"\}"),
        (TokenType::Multiply, r"\*"),
        (TokenType::Divide, r"/"),
        (TokenType::Mismatch, r"."),
    ];

    let pattern: String = token_specs
        .iter()
        .map(|(t, r)| format!(r"(?P<{}>{})", token_type_to_name(t), r))
        .collect::<Vec<_>>()
        .join("|");

    let re = Regex::new(&pattern).map_err(|e| format!("Regex error: {}", e))?;
    let mut tokens = Vec::new();
    let mut position = 0;
    let mut line = 1;

    while position < code.len() {
        if code[position..].starts_with(|c: char| c.is_whitespace()) {
            let c = code.chars().nth(position).unwrap();
            if c == '\n' {
                line += 1;
            }
            position += 1;
            continue;
        }

        let Some(captures) = re.captures(&code[position..]) else {
            return Err(format!(
                "Unexpected character at position {}",
                position
            ));
        };

        let (token_type, value) = token_specs
            .iter()
            .find_map(|(t, _)| {
                captures
                    .name(token_type_to_name(t))
                    .map(|m| (t.clone(), m.as_str().to_string()))
            })
            .ok_or_else(|| format!("Unexpected token at position {}", position))?;

        if token_type == TokenType::Mismatch {
            return Err(format!(
                "Unexpected character '{}' at position {}",
                value, position
            ));
        }

        tokens.push(Token {
            type_: token_type,
            value: value.clone(),
            line,
            position,
        });

        position += value.len();
    }

    Ok(tokens)
}

fn token_type_to_name(t: &TokenType) -> &str {
    match t {
        TokenType::Let => "LET",
        TokenType::Print => "PRINT",
        TokenType::Method => "METHOD",
        TokenType::Fun => "FUN",
        TokenType::Back => "BACK",
        TokenType::Identifier => "IDENTIFIER",
        TokenType::Number => "NUMBER",
        TokenType::String => "STRING",
        TokenType::Plus => "PLUS",
        TokenType::Minus => "MINUS",
        TokenType::Assign => "ASSIGN",
        TokenType::Semicolon => "SEMICOLON",
        TokenType::LParen => "LPAREN",
        TokenType::RParen => "RPAREN",
        TokenType::LBrace => "LBRACE",
        TokenType::RBrace => "RBRACE",
        TokenType::Multiply => "MULTIPLY",
        TokenType::Divide => "DIVIDE",
        TokenType::Mismatch => "MISMATCH",
    }
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    Let {
        name: String,
        value: Box<ASTNode>,
    },
    Print {
        value: Box<ASTNode>,
    },
    Method {
        name: String,
        body: Vec<ASTNode>,
        local_symbol_table: HashMap<String, i32>,
        return_value: Option<String>,
    },
    Fun {
        name: String,
        body: Vec<ASTNode>,
    },
    Back {
        value: String,
    },
    FunctionCall {
        name: String,
        args: Vec<ASTNode>,
    },
    Identifier {
        name: String,
    },
    Number {
        value: String,
    },
    String {
        value: String,
    },
    Assign {
        name: String,
        value: Box<ASTNode>,
    },
}

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn eat(&mut self, expected_type: TokenType) -> Result<(), String> {
        if let Some(token) = self.current_token() {
            if token.type_ == expected_type {
                self.pos += 1;
                Ok(())
            } else {
                Err(format!(
                    "Expected {:?}, got {:?}",
                    expected_type, token.type_
                ))
            }
        } else {
            Err(format!("Expected {:?}, got EOF", expected_type))
        }
    }

    fn parse_expression(&mut self) -> Result<ASTNode, String> {
        let token = self.current_token().ok_or("Unexpected EOF in expression")?;

        match token.type_ {
            TokenType::Identifier => {
                let name = token.value.clone();
                self.eat(TokenType::Identifier)?;

                if let Some(next_token) = self.current_token() {
                    if next_token.type_ == TokenType::LParen {
                        self.parse_function_call(name)
                    } else {
                        Ok(ASTNode::Identifier { name })
                    }
                } else {
                    Ok(ASTNode::Identifier { name })
                }
            }
            TokenType::Number => {
                let value = token.value.clone();
                self.eat(TokenType::Number)?;
                Ok(ASTNode::Number { value })
            }
            TokenType::String => {
                let value = token.value.clone();
                self.eat(TokenType::String)?;
                Ok(ASTNode::String { value })
            }
            _ => Err(format!(
                "Unexpected token {:?} in expression",
                token.type_
            )),
        }
    }

    fn parse_let(&mut self) -> Result<ASTNode, String> {
        self.eat(TokenType::Let)?;

        let ident_token = self.current_token().ok_or("Expected identifier after let")?;
        if ident_token.type_ != TokenType::Identifier {
            return Err("Expected identifier after let".to_string());
        }
        let name = ident_token.value.clone();
        self.eat(TokenType::Identifier)?;

        self.eat(TokenType::Assign)?;
        let value = self.parse_expression()?;
        self.eat(TokenType::Semicolon)?;

        Ok(ASTNode::Let {
            name,
            value: Box::new(value),
        })
    }

    fn parse_print(&mut self) -> Result<ASTNode, String> {
        self.eat(TokenType::Print)?;
        self.eat(TokenType::LParen)?;
        let expr = self.parse_expression()?;
        self.eat(TokenType::RParen)?;
        self.eat(TokenType::Semicolon)?;

        Ok(ASTNode::Print {
            value: Box::new(expr),
        })
    }

    fn parse_function_call(&mut self, func_name: String) -> Result<ASTNode, String> {
        self.eat(TokenType::LParen)?;
        let mut args = Vec::new();

        while let Some(token) = self.current_token() {
            if token.type_ == TokenType::RParen {
                break;
            }

            if token.type_ == TokenType::Semicolon {
                self.eat(TokenType::Semicolon)?;
                continue;
            }

            let arg = self.parse_expression()?;
            args.push(arg);

            if let Some(next_token) = self.current_token() {
                if next_token.type_ == TokenType::Semicolon {
                    self.eat(TokenType::Semicolon)?;
                } else if next_token.type_ != TokenType::RParen {
                    self.eat(TokenType::Plus)?;
                }
            }
        }

        self.eat(TokenType::RParen)?;
        Ok(ASTNode::FunctionCall { name: func_name, args })
    }

    fn parse_method(&mut self) -> Result<ASTNode, String> {
        self.eat(TokenType::Method)?;

        let ident_token = self.current_token().ok_or("Expected method name")?;
        if ident_token.type_ != TokenType::Identifier {
            return Err("Expected method name".to_string());
        }
        let name = ident_token.value.clone();
        self.eat(TokenType::Identifier)?;

        self.eat(TokenType::LBrace)?;
        let mut body = Vec::new();

        while let Some(token) = self.current_token() {
            if token.type_ == TokenType::RBrace {
                break;
            }

            if token.type_ == TokenType::Semicolon {
                self.eat(TokenType::Semicolon)?;
                continue;
            }

            let stmt = self.parse_statement()?;
            body.push(stmt);
        }

        self.eat(TokenType::RBrace)?;

        if let Some(token) = self.current_token() {
            if token.type_ == TokenType::Semicolon {
                self.eat(TokenType::Semicolon)?;
            }
        }

        Ok(ASTNode::Method {
            name,
            body,
            local_symbol_table: HashMap::new(),
            return_value: None,
        })
    }

    fn parse_back(&mut self) -> Result<ASTNode, String> {
        self.eat(TokenType::Back)?;
        let expr = self.parse_expression()?;
        self.eat(TokenType::Semicolon)?;

        let value = match expr {
            ASTNode::Identifier { name } => name,
            ASTNode::Number { value } => value,
            _ => return Err("Invalid expression in back statement".to_string()),
        };

        Ok(ASTNode::Back { value })
    }

    fn parse_fun(&mut self) -> Result<ASTNode, String> {
        self.eat(TokenType::Fun)?;

        let ident_token = self.current_token().ok_or("Expected function name")?;
        if ident_token.type_ != TokenType::Identifier {
            return Err("Expected function name".to_string());
        }
        let name = ident_token.value.clone();
        self.eat(TokenType::Identifier)?;

        // Handle optional parentheses for parameters (even if empty)
        if let Some(token) = self.current_token() {
            if token.type_ == TokenType::LParen {
                self.eat(TokenType::LParen)?;
                self.eat(TokenType::RParen)?;
            }
        }

        self.eat(TokenType::LBrace)?;
        let mut body = Vec::new();

        while let Some(token) = self.current_token() {
            if token.type_ == TokenType::RBrace {
                break;
            }

            if token.type_ == TokenType::Semicolon {
                self.eat(TokenType::Semicolon)?;
                continue;
            }

            let stmt = self.parse_statement()?;
            body.push(stmt);
        }

        self.eat(TokenType::RBrace)?;

        if let Some(token) = self.current_token() {
            if token.type_ == TokenType::Semicolon {
                self.eat(TokenType::Semicolon)?;
            }
        }

        Ok(ASTNode::Fun { name, body })
    }

    fn parse_statement(&mut self) -> Result<ASTNode, String> {
        let token = self.current_token().ok_or("Unexpected EOF")?;

        match token.type_ {
            TokenType::Let => self.parse_let(),
            TokenType::Print => self.parse_print(),
            TokenType::Method => self.parse_method(),
            TokenType::Fun => self.parse_fun(),
            TokenType::Back => self.parse_back(),
            TokenType::Identifier => {
                if self.pos + 1 < self.tokens.len() && self.tokens[self.pos + 1].type_ == TokenType::Assign {
                    let name = token.value.clone();
                    self.eat(TokenType::Identifier)?;
                    self.eat(TokenType::Assign)?;
                    let value = self.parse_expression()?;
                    self.eat(TokenType::Semicolon)?;

                    Ok(ASTNode::Assign {
                        name,
                        value: Box::new(value),
                    })
                } else {
                    let func_name = token.value.clone();
                    let func_call = self.parse_function_call(func_name)?;
                    self.eat(TokenType::Semicolon)?;
                    Ok(func_call)
                }
            }
            _ => Err(format!("Unexpected token {:?}", token.type_)),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut statements = Vec::new();

        while self.pos < self.tokens.len() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }

        Ok(statements)
    }
}

pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, ASTNode>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, nodes: &mut [ASTNode]) -> Result<(), String> {
        for node in nodes {
            self.analyze_node(node)?;
        }
        Ok(())
    }

    fn analyze_node(&mut self, node: &mut ASTNode) -> Result<(), String> {
        match node {
            ASTNode::Method { name, body, local_symbol_table, return_value } => {
                for stmt in body {
                    self.analyze_node(stmt)?;

                    match stmt {
                        ASTNode::Let { name, value } => {
                            if let ASTNode::Number { value: num_val } = &**value {
                                let num = num_val.parse().map_err(|_| format!("Invalid number: {}", num_val))?;
                                local_symbol_table.insert(name.clone(), num);
                            }
                        }
                        ASTNode::Back { value } => {
                            *return_value = Some(value.clone());
                        }
                        _ => {}
                    }
                }

                self.symbol_table.insert(name.clone(), node.clone());
                Ok(())
            }
            ASTNode::Fun { name: _, body } => {
                for stmt in body {
                    self.analyze_node(stmt)?;
                }
                Ok(())
            }
            ASTNode::FunctionCall { name, args } => {
                if !self.symbol_table.contains_key(name) {
                    return Err(format!("Undefined function: {}", name));
                }

                if let ASTNode::Method { return_value, .. } = &self.symbol_table[name] {
                    if return_value.is_none() {
                        return Err(format!("Function {} has no return value", name));
                    }
                } else {
                    return Err(format!("{} is not a function", name));
                }

                for arg in args {
                    self.analyze_node(arg)?;
                }

                Ok(())
            }
            ASTNode::Assign { name, value } => {
                self.analyze_node(value)?;

                if let ASTNode::Identifier { name: ident_name } = &**value {
                    if !self.symbol_table.contains_key(ident_name) {
                        return Err(format!("Undefined variable: {}", ident_name));
                    }
                }

                self.symbol_table.insert(name.clone(), ASTNode::Identifier { name: name.clone() });
                Ok(())
            }
            ASTNode::Let { name, value } => {
                self.analyze_node(value)?;
                self.symbol_table.insert(name.clone(), ASTNode::Identifier { name: name.clone() });
                Ok(())
            }
            ASTNode::Print { value } => {
                self.analyze_node(value)?;
                Ok(())
            }
            ASTNode::Back { .. } => Ok(()),
            ASTNode::Identifier { name } => {
                if !self.symbol_table.contains_key(name) {
                    return Err(format!("Undefined identifier: {}", name));
                }
                Ok(())
            }
            ASTNode::Number { .. } => Ok(()),
            ASTNode::String { .. } => Ok(()),
        }
    }
}

pub fn generate_code(nodes: &[ASTNode]) -> Result<String, String> {
    let mut code = String::new();
    let mut has_main = false;

    for node in nodes {
        if let ASTNode::Fun { name, .. } = node {
            if name == "main" {
                has_main = true;
            }
        }
        code.push_str(&generate_node_code(node)?);
        code.push('\n');
    }

    if !has_main {
        code.push_str("\nfn main() {\n}\n");
    }

    Ok(code)
}

fn generate_node_code(node: &ASTNode) -> Result<String, String> {
    match node {
        ASTNode::Let { name, value } => {
            Ok(format!("let {} = {};", name, generate_node_code(value)?))
        }
        ASTNode::Print { value } => {
            let expr = generate_node_code(value)?;
            match **value {
                ASTNode::String { .. } => Ok(format!("print!({});", expr)),
                _ => Ok(format!("print!(\"{{}}\", {});", expr)),
            }
        }
        ASTNode::Method { name, body, .. } => {
            let mut method_code = format!("fn {}() -> i32 {{\n", name);

            for stmt in body {
                let stmt_code = generate_node_code(stmt)?;
                method_code.push_str(&format!("    {}\n", stmt_code));
            }

            let has_return = body.iter().any(|n| matches!(n, ASTNode::Back { .. }));
            if !has_return {
                method_code.push_str("    return 0;\n");
            }

            method_code.push('}');
            Ok(method_code)
        }
        ASTNode::Fun { name, body } => {
            let mut fun_code = format!("fn {}() {{\n", name);

            for stmt in body {
                let stmt_code = generate_node_code(stmt)?;
                fun_code.push_str(&format!("    {}\n", stmt_code));
            }

            fun_code.push('}');
            Ok(fun_code)
        }
        ASTNode::Back { value } => {
            Ok(format!("return {};", value))
        }
        ASTNode::FunctionCall { name, args } => {
            let args_code: Vec<String> = args.iter()
                .map(|arg| generate_node_code(arg))
                .collect::<Result<_, _>>()?;

            Ok(format!("{}({})", name, args_code.join(", ")))
        }
        ASTNode::Identifier { name } => {
            Ok(name.clone())
        }
        ASTNode::Number { value } => {
            Ok(value.clone())
        }
        ASTNode::String { value } => {
            Ok(value.clone())
        }
        ASTNode::Assign { name, value } => {
            Ok(format!("{} = {};", name, generate_node_code(value)?))
        }
    }
}

fn main() -> Result<(), String> {
    let code = "fun main() { let x = \"5\"; print(x); }";

    println!("Original code:\n{}", code);

    let tokens = lexer(code)?;
    println!("\nTokens:");
    for token in &tokens {
        println!("{:?}", token);
    }

    let mut parser = Parser::new(tokens);
    let mut ast = parser.parse()?;
    println!("\nAST:");
    for node in &ast {
        println!("{:?}", node);
    }

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&mut ast)?;

    let generated_code = generate_code(&ast)?;
    println!("\nGenerated Code:");
    println!("{}", generated_code);

    std::fs::write("generated.rs", &generated_code)
        .map_err(|e| format!("Failed to write generated code: {}", e))?;
    println!("\nGenerated code written to 'generated.rs'");

    let compile_output = std::process::Command::new("rustc")
        .arg("generated.rs")
        .arg("-o")
        .arg(".\\generated_bin.exe")
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    if !compile_output.status.success() {
        let err_msg = String::from_utf8_lossy(&compile_output.stderr);
        return Err(format!("Compilation failed: {}", err_msg));
    }
    println!("\nCompilation successful. Output binary: 'generated_bin.exe'");

    let run_output = std::process::Command::new(".\\generated_bin.exe")
        .output()
        .map_err(|e| format!("Failed to run binary: {}", e))?;

    if !run_output.status.success() {
        let err_msg = String::from_utf8_lossy(&run_output.stderr);
        return Err(format!("Execution failed: {}", err_msg));
    }

    let output = String::from_utf8_lossy(&run_output.stdout);
    println!("\nProgram output:\n{}", output);

    Ok(())
}