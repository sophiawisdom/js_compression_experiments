fn print_tabs(depth: i32) {
    for i in 0..depth {
        print!("\t");
    }
}

fn print_object_literal(depth: i32, obj: ObjectLit) {
    for propOrSpread in obj.props {
        print_tabs(depth);
        match propOrSpread {
            swc_ecma_ast::PropOrSpread::Spread(spread) => {
                println!("Got a spread!");
            }
            swc_ecma_ast::PropOrSpread::Prop(prop) => {
                print!("got a prop: ");
                match *prop {
                    swc_ecma_ast::Prop::Shorthand(ident) => {
                        println!("shorthand: {}", ident.sym);
                    }
                    swc_ecma_ast::Prop::KeyValue(val) => {
                        print!("key: ");
                        match val.key {
                            swc_ecma_ast::PropName::Ident(id) => {
                                println!("sym {}", id.sym);
                            }
                            swc_ecma_ast::PropName::Str(str) => {
                                println!("str {}", str.value);    
                            }
                            _ => {
                                println!("{:?}", val.key);
                            }
                        }
                        recurse_expr(depth+1, *val.value);
                    }
                    _ => {
                        println!("unknown prop");
                    }
                }
            }
        }
    }
}

fn recurse_expr(depth: i32, expr: swc_ecma_ast::Expr) {
    print_tabs(depth);

    match expr {
        swc_ecma_ast::Expr::Array(array) => {
            println!("got array");
            for val in array.elems {
                match val {
                    Some(v) => {
                        recurse_expr(depth+1, *v.expr);
                    }
                    None => {
                        
                    }
                }
            }
        }
        swc_ecma_ast::Expr::Arrow(arrow) => {
            println!("got arrow");
        }
        swc_ecma_ast::Expr::Assign(assign) => {
            println!("got assign");
        }
        swc_ecma_ast::Expr::Await(awai) => {
            println!("got await");
        }
        swc_ecma_ast::Expr::Bin(bin) => {
            println!("got bin: op is {}", bin.op.as_str());
            recurse_expr(depth+1, *bin.left);
            recurse_expr(depth+1, *bin.right);
        }
        swc_ecma_ast::Expr::Call(cl) => {
            println!("Got {} args.", cl.args.len());
            for arg in cl.args {
                recurse_expr(depth+1, *arg.expr);
            }
            print_tabs(depth);
            match cl.callee {
                swc_ecma_ast::Callee::Super(s) => {
                    println!("super call!");
                }
                swc_ecma_ast::Callee::Import(imp) => {
                    println!("import call!");
                }
                swc_ecma_ast::Callee::Expr(exp) => {
                    println!("Got expr call!");
                    recurse_expr(depth+1,*exp);
                }
            }

        }
        swc_ecma_ast::Expr::Class(cls) => {
            println!("got class");
        }
        swc_ecma_ast::Expr::Cond(cond) => {
            println!("got cond");
        }
        swc_ecma_ast::Expr::Fn(function) => {
            println!("Got fn!");
        }
        swc_ecma_ast::Expr::Ident(ident) => {
            println!("got ident");
        }
        swc_ecma_ast::Expr::Invalid(invalid) => {
            println!("got invalid");
        }
        swc_ecma_ast::Expr::Lit(lit) => {
            match lit {
                swc_ecma_ast::Lit::Str(str) => {
                    println!("got string literal {}", str.value);
                }
                swc_ecma_ast::Lit::Num(num) => {
                    println!("Got number literal {}", num.value);
                }
                _ => {
                    println!("got literal {:?}", lit);
                }
            }
        }
        swc_ecma_ast::Expr::Member(mem) => {
            println!("got member. span {} {}", mem.span.lo.0, mem.span.hi.0);
        }
        swc_ecma_ast::Expr::MetaProp(meta) => {
            println!("got meta");
        }
        swc_ecma_ast::Expr::New(new) => {
            println!("got new");
        }
        swc_ecma_ast::Expr::Object(obj) => {
            println!("got object literal: ");
            print_object_literal(depth+1, obj);
        }
        swc_ecma_ast::Expr::OptChain(optchain) => {
            println!("got optchain");
        }
        swc_ecma_ast::Expr::Paren(paren) => {
            println!("got paren. span {} {}", paren.span.lo.0, paren.span.hi.0);
            recurse_expr(depth+1, *paren.expr);
        }
        swc_ecma_ast::Expr::PrivateName(private) => {
            println!("got private");
        }
        swc_ecma_ast::Expr::Seq(seq) => {
            println!("got seq. it's got {} values", seq.exprs.len());
            for expr in seq.exprs {
                recurse_expr(depth+1, *expr);
            }
        }
        swc_ecma_ast::Expr::SuperProp(prop) => {
            println!("got super prop");
        }
        swc_ecma_ast::Expr::TaggedTpl(taggedtpl) => {
            println!("got tagged tpl");
        }
        swc_ecma_ast::Expr::This(this) => {
            println!("got this");
        }
        swc_ecma_ast::Expr::Tpl(tpl) => {
            println!("got tpl");
        }
        swc_ecma_ast::Expr::Unary(unary) => {
            println!("got unary");
        }
        swc_ecma_ast::Expr::Update(update) => {
            println!("got update");
        }
        swc_ecma_ast::Expr::Yield(yld) => {
            println!("got yield");
        }
        _ => {
            println!("Got unknown expr type");
        }
    }
}

fn recurse(statement: Stmt) {
    match statement {
        swc_ecma_ast::Stmt::Block(block) => {
            println!("Got block type!");
        }
        swc_ecma_ast::Stmt::Decl(decl) => {
            println!("Got decl type!");
        }
        swc_ecma_ast::Stmt::Expr(expr) => {
            recurse_expr(0, *expr.expr);
        }
        swc_ecma_ast::Stmt::Block(block) => {
            println!("Got block type!");
        }
        _ => {
            println!("Got unknown type!");
        }
    }
}