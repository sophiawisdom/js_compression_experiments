use std::collections::HashMap;
use std::{io::Read};

use simhash::hamming_distance;
use swc_common::sync::Lrc;
use swc_common::{SourceMap};
use swc_ecma_ast::{Stmt, ObjectLit};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::VisitMut;
use swc_common::Spanned;

use rand::thread_rng;
use rand::Rng;
use rand::prelude::SliceRandom;

mod traveling_salesman;

fn getHiLo(node: impl Spanned) -> (u32, u32) {
    let span = node.span();
    (span.lo.0, span.hi.0)
}

struct Compressor<'a> {
    // (u32, u32) char span -> 64 bit hash
    simhashMap: std::collections::HashMap<(u32, u32), u64>,
    // 64 bit hash -> (u32, u32) char span
    reverseSimhashes: std::collections::HashMap<u64, (u32, u32)>,
    simhashes: Vec<u64>,
    cm: &'a Lrc<SourceMap>,

    runsWithNoChanges: u32,

    distanceMatrix: Vec<Vec<f64>>,
}

fn window_distance(simhashes: &Vec<u64>, index: usize, window: usize) -> u32 {
    let mut total_distance = 0;
    let index_simhash = simhashes[index];
    let min_value = std::cmp::max((index as isize) - (window as isize), 0) as usize;
    let max_value = std::cmp::min(index + window, simhashes.len());
    for i in min_value..max_value {
        total_distance += hamming_distance(simhashes[i], index_simhash);
    }
    total_distance
}

impl<'a> VisitMut for Compressor<'a> {
    fn visit_mut_object_lit(&mut self, lit: &mut ObjectLit) {
        if self.simhashMap.len() == 0 {
            println!("Calculating simhashes");
            for prop in &lit.props {
                let buf = ast_bytes(prop, self.cm);
                let str = String::from_utf8(buf).expect("invalid utf8 character detected");
                let hashed = simhash::simhash(&str);
                self.simhashMap.insert(getHiLo(prop), hashed);
                self.reverseSimhashes.insert(hashed, getHiLo(prop));
                self.simhashes.push(hashed);
            }

            let simhash_len = self.simhashes.len();
            self.distanceMatrix = Vec::with_capacity(simhash_len);
            for columnHash in &self.simhashes {
                let mut array = Vec::with_capacity(simhash_len);
                for rowHash in &self.simhashes {
                    array.push(simhash::hamming_distance(*columnHash, *rowHash) as f64);
                }
                self.distanceMatrix.push(array);
            }
        }

        /*
        let mut rng = thread_rng();
        let mut salesman = traveling_salesman::TravellingSalesman{distance_matrix: &self.distanceMatrix, rng: &mut rng};
        let result = metaheuristics::hill_climbing::solve(&mut salesman,  time::Duration::new(30, 0));
        println!("got distance {}", traveling_salesman::get_route_distance(&self.distanceMatrix, &result.route));
        let mut new_props: Vec<swc_ecma_ast::PropOrSpread> = vec![];
        let mut crap = std::collections::HashMap::new();
        for i in &lit.props {
            crap.insert(getHiLo(i), i.clone());
        }
        for i in result.route {
            let hsh = self.simhashes[i];
            let hshSpan = self.reverseSimhashes.get(&hsh).unwrap();
            let prp = crap.get(hshSpan).unwrap().clone();
            new_props.push(prp);
            // print!("{} ", i);
        }
        lit.props = new_props;
        */

        let mut rng = thread_rng();
        let num_props = lit.props.len();
        let mut changes = 0;
        for _ in 0..100 {
            let first = rng.gen_range(0..num_props);
            let second = rng.gen_range(0..num_props);

            let cur_first_distance = window_distance(&self.simhashes, first, 50);
            let cur_second_distance = window_distance(&self.simhashes, second, 50);
            let cur_distance = cur_first_distance + cur_second_distance;

            self.simhashes.swap(first, second);

            let new_first_distance = window_distance(&self.simhashes, first, 50);
            let new_second_distance = window_distance(&self.simhashes, second, 50);
            let new_distance = new_first_distance + new_second_distance;

            if new_distance < cur_distance {
                lit.props.swap(first, second);
                changes += 1;
            } else {
                // unswap em
                self.simhashes.swap(first, second);
            }
        }
        if changes > 0 {
            self.runsWithNoChanges = 0;
        } else {
            self.runsWithNoChanges += 1;
        }
        if self.runsWithNoChanges >= 10 {
            println!("Shufflin!");
            self.runsWithNoChanges = 0;
            lit.props.shuffle(&mut thread_rng());
            let mut newSimhashes: Vec<u64> = vec![];
            for prop in &lit.props {
                let simhashForProp = self.simhashMap.get(&getHiLo(prop)).unwrap();
                newSimhashes.push(*simhashForProp);
            }
            self.simhashes = newSimhashes;
        }
        // println!("Made {} changes", changes);
        // lit.props.shuffle(&mut thread_rng());
    }
}

fn ast_bytes(node: &impl swc_ecma_codegen::Node, cm: &Lrc<SourceMap>) -> Vec<u8> {
    let mut buf = vec![];
    {
        let mut wr = Box::new(swc_ecma_codegen::text_writer::JsWriter::with_target(
            cm.clone(),
            "\n",
            &mut buf,
            None,
            swc_ecma_ast::EsVersion::Es2022,
        )) as Box<dyn swc_ecma_codegen::text_writer::WriteJs>;

        wr = Box::new(swc_ecma_codegen::text_writer::omit_trailing_semi(wr));

        let mut emitter = swc_ecma_codegen::Emitter {
            cfg: swc_ecma_codegen::Config { minify: true },
            comments: None,
            cm: cm.clone(),
            wr,
        };

        node.emit_with(&mut emitter).unwrap();
    }
    buf
}

fn ast_bytes_compressed(node: &Stmt, cm: &Lrc<SourceMap>, quality: u32) -> usize {
    let before_fold = std::time::SystemTime::now();
    let buf = ast_bytes(node, cm);
    let after_fold = std::time::SystemTime::now();

    let len = buf.len();
    let mut input = brotli::CompressorReader::new(buf.as_slice(), len, quality, len as u32);
    let mut compressed_bytes = vec![];

    let val = match input.read_to_end(&mut compressed_bytes) {
        Ok(val) => val,
        Err(e) => panic!("got error compressing {}", e)
    };
    let after_compression = std::time::SystemTime::now();
    /*
    println!("serialization to AST took {} millis, compression {} millis",
        after_fold.duration_since(before_fold).unwrap().as_millis(),    
        after_compression.duration_since(after_fold).unwrap().as_millis(),
    );
    */
    val
}

fn main() {

    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm
            .load_file(std::path::Path::new("main.js"))
            .expect("failed to load .js");
    
        let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        println!("Got parser error {:?}", e);
    }

    let module = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            println!("Got parser error {:?}", e);
        })
        .expect("failed to parser module");

    let mut stmt = None;
    for item in module.body {
        if item.is_module_decl() {
            println!("Got worthless decl!");
            continue;
        }
        stmt = Some(item.expect_stmt());
        break;
    }
    let initial_size = ast_bytes_compressed(&stmt.unwrap(), &cm, 9) as u32;
    println!("Initial size is {}", initial_size);


    // let before_bytes_compressed = ast_bytes_compressed(&statement_to_modify, &cm, 9);
    // let mut sizes = vec![];
    let min_size = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(initial_size));
    let total_processed = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let mut handles = vec![];
    for i in 0..1 {
        let minsize_clone = std::sync::Arc::clone(&min_size);
        let total_processed_clone = std::sync::Arc::clone(&total_processed);
        handles.push(std::thread::spawn(move|| {
            let cm: Lrc<SourceMap> = Default::default();

            let fm = cm
                 .load_file(std::path::Path::new("main.js"))
                 .expect("failed to load .js");
            
                let lexer = Lexer::new(
                // We want to parse ecmascript
                Syntax::Es(Default::default()),
                // EsVersion defaults to es5
                Default::default(),
                StringInput::from(&*fm),
                None,
            );
        
            let mut parser = Parser::new_from(lexer);
        
            for e in parser.take_errors() {
                println!("Got parser error {:?}", e);
            }
        
            let module = parser
                .parse_module()
                .map_err(|e| {
                    // Unrecoverable fatal error occurred
                    println!("Got parser error {:?}", e);
                })
                .expect("failed to parser module");
        
            let mut statement_to_modify = None;
            for item in module.body {
                if item.is_module_decl() {
                    println!("Got worthless decl!");
                    continue;
                }
                statement_to_modify = Some(item.expect_stmt());
                break;
            }
        
            let mut statement_to_modify = statement_to_modify.unwrap();

            let mut threadlocal_compress = Compressor{
                simhashMap: HashMap::new(), reverseSimhashes: HashMap::new(), simhashes: vec![], cm: &cm, runsWithNoChanges: 0, distanceMatrix: vec![]
            };

            for _ in 0..100000 {
                // randomly mutate
                swc_ecma_visit::visit_mut_stmt(&mut threadlocal_compress, &mut statement_to_modify);
                let size = ast_bytes_compressed(&statement_to_modify, &cm, 9) as u32;
                let prev_val = minsize_clone.fetch_min(size, std::sync::atomic::Ordering::AcqRel);
                println!("Compressed size is {}", size);
                if size < prev_val {
                    println!("Got new size smaller than min on thread {}!!! {}", i, size);
                }
                if total_processed_clone.fetch_add(1, std::sync::atomic::Ordering::AcqRel) % 1000 == 0 {
                    println!("Have now processed {} permutations", total_processed_clone.load(std::sync::atomic::Ordering::Acquire));
                }
            }   
        }))
    }

    for i in handles {
        i.join().unwrap();
    }
    // sizes.sort_unstable();
}