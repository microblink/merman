use clap::Parser;

mod graph;
mod layouting;

#[derive(clap::Parser, Debug)]
struct Args {
    /// Output path.
    /// It can be a .json or a markdown file.
    /// If not specified and the input file is a markdown file, the input file will be modified inplace.
    /// If not specified and the input file is a .json file, svg output will be written to stdout.
    #[clap(short, long)]
    output: Option<String>,

    /// Path to the input .md or .json file
    path: String,
}

trait FindFrom {
    fn find_from(&self, start: usize, pattern: &str) -> Option<usize>;
}

impl FindFrom for str {
    fn find_from(&self, start: usize, pattern: &str) -> Option<usize> {
        self[start..].find(pattern).map(|p| p + start)
    }
}

fn generate_svg(content: &str) -> String {
    let g = graph::Graph::from_str(content).expect("Could not parse graph");

    let sort_order = g.reverse_topological_sort().expect("Could not sort!");

    layouting::to_svg(&g, &sort_order, &layouting::DEFAULT_STYLE)
}

fn transform_markdown(content: &mut String) {
    const START_STR: &str = "```merman\n";
    const END_STR: &str = "```\n";

    let mut current_index = 0usize;

    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    while let Some(start_index) = content.find_from(current_index, START_STR) {
        if let Some(end_index) = content.find_from(start_index + START_STR.len(), END_STR) {
            let svg = generate_svg(&content[start_index + START_STR.len()..end_index]);

            replacements.push((start_index, end_index + END_STR.len(), svg));

            current_index = end_index + END_STR.len();
        } else {
            // Unmatched code, just return
            break;
        }
    }

    replacements.reverse();

    for (start_index, end_index, svg) in replacements {
        content.replace_range(start_index..end_index, &svg);
    }
}

fn main() {
    let args = Args::parse();

    let is_json = args.path.ends_with(".json");

    let mut content = std::fs::read_to_string(&args.path).expect("Could not read input file!");

    if is_json {
        content = generate_svg(&content);
    } else {
        transform_markdown(&mut content);
    }

    match (is_json, args.output) {
        (_, Some(output_path)) => {
            // Write to the output file
            std::fs::write(output_path, content).expect("Could not write to output path");
        }
        (false, None) => {
            // In case of markdown transformation, replace the markdown file with new content
            std::fs::write(&args.path, content).expect("Could not replace input file content!");
        }
        (true, None) => {
            // Just write to stdout
            println!("{content}");
        }
    }
}
