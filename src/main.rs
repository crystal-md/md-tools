#[macro_use]
extern crate clap;

use std::path::Path;

mod ffi;
mod xyz_parser;
mod text_parser;

mod ans;

fn main() {
    use clap::App;

    let yml = load_yaml!("cli.yaml");
    let matches = App::from(yml)
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .setting(clap::AppSettings::ColoredHelp)
        .version(crate_version!())
        .author(crate_authors!())
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("conv") {
        parse_convert(matches);
        return;
    }
    if let Some(ref matches) = matches.subcommand_matches("ans") {
        parse_ans(matches);
        return;
    }
    println!("No subcommand is used");
}

fn parse_convert(matches: &&clap::ArgMatches) {
    let ranks = matches.value_of_t("ranks").unwrap_or(0);
    if ranks <= 0 {
        println!("unsupported ranks value.");
        return;
    }
    let format = matches.value_of("format").unwrap();
    if !(format == "xyz" || format == "text" || format == "db" || format == "def") {
        println!("unsupported format {}.", format);
        return;
    }

    let dry_run = matches.is_present("dry");
    let output = matches.value_of("output").unwrap();
    let mut input_files = Vec::new();
    if let Some(arg_input) = matches.values_of("input") {
        for in_file in arg_input {
            input_files.push(in_file);
        }
    }

    if input_files.len() == 0 {
        println!("no matching input files");
        return;
    } else if input_files.len() == 1 {
        if !dry_run {
            mk_parse(format, ranks, input_files[0], output);
        }
        println!("file {} converted, saved at {}", input_files[0], output);
    } else {
        for input_file in input_files {
            let input_path = Path::new(input_file);

            println!("converting file {}", input_file);
            let output_suffix = input_path.file_name().unwrap().to_str().unwrap();
            let output_file_path = format!("{}.{}", output, output_suffix);
            if !dry_run {
                mk_parse(format, ranks, input_file, output_file_path.as_str());
            }
            println!("file {} converted, saved at {}", input_file, output_file_path.as_str());
        }
    }
}

fn parse_ans(matches: &&clap::ArgMatches) {
    let output = matches.value_of("output").unwrap();
    let mut input_files = Vec::new();
    if let Some(arg_input) = matches.values_of("input") {
        for in_file in arg_input {
            input_files.push(in_file);
        }
    }

    let mut box_size: Vec<u64> = Vec::new();
    if matches.is_present("box-size") {
        for l_size in matches.values_of_t::<u64>("box-size").unwrap() {
            box_size.push(l_size);
        }
    }
    let box_config = ans::analysis::BoxConfig {
        box_size,
    };

    let mut input_from_minio: bool = false;
    if matches.is_present("input-from-minio") {
        println!("Now we will read input file from minio or AWS s3");
        input_from_minio = true;
    }

    if input_files.len() == 0 {
        println!("no matching input files");
        return;
    } else if input_files.len() == 1 {
        ans::analysis::voronoy_ans_wrapper(input_files[0], output, input_from_minio, &box_config)
    } else {
        for input_file in input_files {
            let input_path = Path::new(input_file);
            println!("analysing file {}", input_file);
            let output_suffix = input_path.file_name().unwrap().to_str().unwrap();
            let output_file_path = format!("{}.{}", output, output_suffix);
            ans::analysis::voronoy_ans_wrapper(input_file, output_file_path.as_str(), input_from_minio, &box_config);
            println!("file {} analysis, saved at {}", input_file, output_file_path.as_str());
        }
    }
    // todo method
    // todo, now only xyz format input is supported
}

fn mk_parse(format: &str, ranks: u32, input: &str, output: &str) {
    match format {
        "xyz" => {
            ffi::parse(input, output, ranks, xyz_parser::new_parser(output));
        }
        "text" => {
            ffi::parse(input, output, ranks, text_parser::new_parser(output));
        }
        "db" => {}
        "def" => {}
        _ => unreachable!()
    }
}
