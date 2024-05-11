//! A main function. Currently doesn't have anything since work on 
//! a databender hasn't started yet.

mod benders;
mod mutations;
mod loaders;
mod configuration;
mod mutations2;
mod config;

use std::error::Error;

use benders::KaBender;
use config::{ioutils::{load_yaml_file, save_bytes_to_file}, parser::{parse_app_cfg, parse_mode, parse_mutations, parse_source, AppCfg}};
use configuration::Configuration;

use rand::{rngs::StdRng, Rng, SeedableRng};
use rayon::prelude::*;

use crate::mutations2::{local::Increment, AreaType, Mutation};

fn main2() {
    // Initialises the configuration for the application.
    let conf = Configuration::from_file("Options.toml");

    conf.verify_config();

    // Initialises the mutation map at the start.
    lazy_static::initialize(&benders::MUTMAP);

    // Retrieves some options from the configuration.
    let loops = conf.get("times")
        .and_then(|times| times.as_int())
        .unwrap_or(&1);

    (0..*loops).into_par_iter().for_each(|i| {
        let bender = KaBender::new(&conf, i.to_string());
        bender.run();
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let yaml = load_yaml_file(args.nth(1).unwrap())?;

    let mode = parse_mode(&yaml);
    let source = parse_source(&yaml);
    let app_cfg = parse_app_cfg(&yaml);

    let mut rng = StdRng::from_entropy();

    let mut file = source.perform();
    let bytesize = file.len();

    println!("file in memory has [{} bytes]", bytesize);

    let mut nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    println!("nums = {:?}", nums);
    let testmut = Increment { by: 2, chunksize: 3 };
    testmut.bend(&mut nums[3..5]);
    println!("nums = {:?}", nums);

    return Ok(());

    for i in 0..app_cfg.output_n {
        let mutations = parse_mutations(&mut rng, &yaml);

        for mut mutation in mutations {
            match mutation.get_type() {
                AreaType::Global => {
                    mutation.as_mut().bend(&mut file)
                },
                AreaType::Local => {
                    let chunksize = mutation.get_chunksize();
                    mutation.as_mut().bend(
                        &mut file[0..rng.gen_range(chunksize, bytesize-chunksize)])
                }
            }
            mutation.as_mut().bend(&mut file);
        }

        println!("saving to file: {}-{}.jpeg", app_cfg.output_path, i);
        save_bytes_to_file(format!("{}-{}.jpg", app_cfg.output_path, i).as_str(), &file)?;
    }

    Ok(())
}