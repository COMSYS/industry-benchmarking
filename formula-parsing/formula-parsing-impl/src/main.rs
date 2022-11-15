//! == FORMULA PARSING PROGRAM ==
//! 
//! - Input: an algorithm file in intermediary format (AST-style yaml)
//! - Output: an algorithm file in atomic format (pruned yaml)
//!     - The output is written to `../data/algo_atomic.yaml`
//!
//!  Author: Eduard Vlad

use std::{fs::File, vec, io::BufReader};
use benchmark::{atomic::Atomic, operation::OperationType};
use parser::Formula;
use serde::{Serialize, Deserialize};

use crate::{parser::{Expression, Literal, TokenType}};
pub mod parser;


static ATOMIC_FILE_OUTPUT: &str = "../data/algo_atomic.yaml";

#[derive(Serialize)]
struct OutputAlgorithms {
    operations: Vec<Atomic>
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct AlgoIO {
    name: String,
    is_kpi: bool,
    op: Formula
}

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Parsing Intermediary Algorithm File in YAML Format!");

    // Get input from user
    let (alg_file, path) = parse_input();

    let forms: Vec<AlgoIO> = serde_yaml::from_reader(BufReader::new(alg_file)).expect("Algorithm format incorrect!");

    // Iterate over all formulas and make them atomic
    let mut init_helper = 0;
    let mut out_op = Vec::new();


    for algo in forms {
        prune_ast(algo.op.0, &mut out_op , &mut init_helper, algo.name, algo.is_kpi);
    }

    // write final output tofile
    if path.len() == 0 {
        write_to_yaml(OutputAlgorithms {operations: out_op}, ATOMIC_FILE_OUTPUT);
    } else {
        write_to_yaml(OutputAlgorithms {operations: out_op}, &path);        
    }
    
}


/// Parse the input argument which is considered to
/// be an Input file for formulas with given format.
fn parse_input() -> (File, String) {
    let args: Vec<_> = std::env::args().collect();
    let mut output_path: String = "".to_string();

    if args.len() < 2 {
        log::warn!("No algorithm file and kpi list file provided!\nUsage: {} <intermediary_algorithm_filename>", args[0]);
        std::process::exit(-1);
    }
    if args.len() == 3 {
        // The output path was provided
        output_path = args[2].clone();
    }
    let alg_fname = std::path::Path::new(&*args[1]);
    if !(alg_fname.exists())
    { log::error!("Could not find the input file! Exiting..."); std::process::exit(-1);}
    
    let alg_file = File::open(&alg_fname).unwrap();

    (alg_file, output_path)
}

// Create a file where the yaml inputs are written into
fn write_to_yaml<T: Serialize>(algo: T, path: &str) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .expect("Couldn't open file");

    serde_yaml::to_writer(file, &algo).expect("Could not write algorithms to file for candidate!");
}

fn prune_ast(traverser: Expression, op_list: &mut Vec<Atomic>, helper_counter: &mut u32, parent_name: String, is_kpi: bool) -> () {
    
    // Helper closures
    let helper_prefix = "he";
        
    let token_to_op = |e: TokenType| -> OperationType {
        match e {
            
            TokenType::OperatorAdd => OperationType::Addition,
            TokenType::OperatorSubtract => OperationType::Subtraction,
            TokenType::OperatorDivide => OperationType::Division,
            TokenType::OperatorMultiply => OperationType::Multiplication,
            TokenType::OperatorPower => OperationType::Power,
            
            TokenType::AbsLine => OperationType::Absolute,
            TokenType::KeywordWurzel => OperationType::Squareroot,
            TokenType::KeywordMin => OperationType::Minima,
            TokenType::KeywordMax => OperationType::Maxima,
            unrecognized => panic!("Trying to translate unrecognized token: {:?}", unrecognized),
        }
    };
    
    //log::info!("TRAVERSER: {:?}",traverser);
    
    match traverser {
        Expression::UnaryExpression(mut exp) => {
            
            match *exp.clone().argument {
                Expression::Literal(lit) => {
                    match lit {
                        Literal::NumericLiteral(e) => {
                            // Define constant
                            let atom_name = format!("{}{:04}",helper_prefix, *helper_counter);
                            *helper_counter +=1;
                            exp.argument = Box::new(Expression::Literal(Literal::Variable(atom_name.clone())));

                            let self_atomic = Atomic::new(parent_name, is_kpi, token_to_op(exp.operator), vec![atom_name.clone()], None);
                            op_list.push(self_atomic);

                            let atom_constant = Atomic::new(atom_name, false, OperationType::DefConst,vec![],Some(e));
                            op_list.push(atom_constant);
                            
                            // No recursive call necessary as we are already finished when only a constant is given!
                        },
                        Literal::Variable(name) => {
                            op_list.push(Atomic::new(parent_name, is_kpi, token_to_op(exp.operator), vec![name], None));
                            log::info!("We reached end in unary!");
                        },
                    }
                },
                sub_expression => {
                    // Prune subtree
                    let atom_name = format!("{}{:04}",helper_prefix, *helper_counter);
                    *helper_counter +=1;
                    exp.argument = Box::new(Expression::Literal(Literal::Variable(atom_name.clone())));

                    // Create self entry of self and push it to the op_list
                    let self_atomic = Atomic::new(parent_name, is_kpi, token_to_op(exp.operator), vec![atom_name.clone()], None);
                    op_list.push(self_atomic);

                    // Recursive call on subtree
                    prune_ast(sub_expression, op_list, helper_counter,atom_name,false);
                },
            }
        },
        Expression::Binary(mut bin_exp) => {
            
            let mut children_names: Vec<String> = vec![];
            // log::info!("OPtype: {}", token_to_op(bin_exp.operator));

            // First left than right
            match *bin_exp.clone().left {
                Expression::Literal(lit) => {
                    match lit {
                        Literal::Variable(name) => {
                            children_names.push(name);
                            //log::info!("We reached left end in binary!");
                        },
                        Literal::NumericLiteral(e) => {
                            // Define constant
                            let atom_left_name = format!("{}{:04}",helper_prefix, *helper_counter);
                            *helper_counter +=1;
                            bin_exp.left = Box::new(Expression::Literal(Literal::Variable(atom_left_name.clone())));

                            // Add to children
                            children_names.push(atom_left_name.clone());

                            let atom_left_constant = Atomic::new(atom_left_name, false, OperationType::DefConst,vec![],Some(e));
                            op_list.push(atom_left_constant);
                        }
                    }
                },
                sub_expression => {
                    // Prune subtree
                    let atom_left_name = format!("{}{:04}",helper_prefix, *helper_counter);
                    *helper_counter +=1;
                    bin_exp.left = Box::new(Expression::Literal(Literal::Variable(atom_left_name.clone())));

                    children_names.push(atom_left_name.clone());

                    // Recursive call on subtree
                    prune_ast(sub_expression, op_list, helper_counter,atom_left_name,false);
                }
            }

            // Now right
            match *bin_exp.clone().right {
                Expression::Literal(lit) => {
                    match lit {
                        Literal::Variable(name) => {
                            children_names.push(name);
                            //log::info!("We reached right end in binary!");
                        },
                        Literal::NumericLiteral(e) => {
                            // Define constant
                            let atom_right_name = format!("{}{:04}",helper_prefix, *helper_counter);
                            *helper_counter +=1;
                            bin_exp.right = Box::new(Expression::Literal(Literal::Variable(atom_right_name.clone())));

                            // Add to children
                            children_names.push(atom_right_name.clone());

                            let atom_right_constant = Atomic::new(atom_right_name, false, OperationType::DefConst,vec![],Some(e));
                            op_list.push(atom_right_constant);
                        }
                    }
                },
                sub_expression => {
                    // Prune subtree
                    let atom_right_name = format!("{}{:04}",helper_prefix, *helper_counter);
                    *helper_counter +=1;
                    bin_exp.right = Box::new(Expression::Literal(Literal::Variable(atom_right_name.clone())));

                    // Add to children
                    children_names.push(atom_right_name.clone());

                    // Recursive call on subtree
                    prune_ast(sub_expression, op_list, helper_counter,atom_right_name,false);
                }
            }

            // Create self entry of self and push it to the op_list
            let self_atomic = Atomic::new(parent_name, is_kpi, token_to_op(bin_exp.operator), children_names, None);
            op_list.push(self_atomic);
        },
        Expression::NAryExpression(nary_expr) => {
            
            let mut children_names: Vec<String> = Vec::new();
            
            for el in nary_expr.clone().operands {
                match *el {
                    Expression::Literal(lit) => {
                        match lit {
                            Literal::NumericLiteral(e) => {
                                // Define constant
                                let atom_name = format!("{}{:04}",helper_prefix, *helper_counter);
                                *helper_counter +=1;
                                //el = Box::new(Expression::Literal(Literal::Variable(atom_name.clone())));
    
                                children_names.push(atom_name.clone());
    
                                let atom_constant = Atomic::new(atom_name, false, OperationType::DefConst,vec![],Some(e));
                                op_list.push(atom_constant);
                                
                                // No recursive call necessary as we are already finished when only a constant is given!
                            },
                            Literal::Variable(name) => {
                                children_names.push(name);
                                //log::info!("We reached (one) end in nary!");
                            },
                        }
                    },
                    sub_expression => {
                        // Prune subtree
                        let atom_name = format!("{}{:04}",helper_prefix, *helper_counter);
                        *helper_counter +=1;
                        // el = Box::new(Expression::Literal(Literal::Variable(atom_name.clone())));
    
                        children_names.push(atom_name.clone());
    
                        // Recursive call on subtree
                        prune_ast(sub_expression, op_list, helper_counter,atom_name,false);
                    },
                }
            }

            // Create self entry of self and push it to the op_list
            let self_atomic = Atomic::new(parent_name, is_kpi, token_to_op(nary_expr.operator), children_names, None);
            op_list.push(self_atomic);
        },
        Expression::Literal(lit) => {
            match lit {
                // This case can only occur if the input is already a literal since in any other case we terminate above
                // Workaround: multiply with 1.0 const or use define const
                Literal::NumericLiteral(e) => {
                    // Define constant from given name
                    let self_atomic = Atomic::new(parent_name, is_kpi, OperationType::DefConst, vec![], Some(e));
                    op_list.push(self_atomic);
                    
                    // No recursive call necessary as we are already finished when only a constant is given!
                },
                Literal::Variable(e) => {
                    // Define constant from given name
                    let self_atomic = Atomic::new(parent_name, is_kpi, OperationType::MultiplicationConst, vec![e], Some(1.0));
                    op_list.push(self_atomic);
                },
            }
        },
    }
}