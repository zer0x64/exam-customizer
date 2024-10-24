use std::{
    fs::{DirBuilder, File, OpenOptions},
    path::PathBuf,
};

use clap::Parser;
use docx_rs::{Docx, Paragraph, Run};

type Result<T> = std::result::Result<T, anyhow::Error>;

#[derive(Parser)]
struct Args {
    students: PathBuf,
    questions: PathBuf,
    output_folder: PathBuf,
}

#[derive(Debug)]
struct Student {
    pub name: String,
    pub group: usize,
    pub scores: Vec<f32>,
}

#[derive(Debug)]
struct Category {
    pub description: String,
    pub questions: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let students = File::open(&args.students)?;
    let questions = File::open(&args.questions)?;

    if args.output_folder.is_dir() {
        std::fs::remove_dir_all(&args.output_folder)?;
    }

    DirBuilder::new().create(&args.output_folder)?;

    let mut students = csv::Reader::from_reader(students);
    let mut questions = csv::Reader::from_reader(questions);

    println!("Parsing students file...");

    let students: Vec<Student> = students
        .records()
        .filter_map(|r| {
            let r = r.ok()?;

            let mut iter = r.iter();
            let name = iter.next()?.to_string();
            let group = iter.next()?.parse().ok()?;

            let scores: Vec<_> = iter.filter_map(|s| s.parse::<f32>().ok()).collect();

            Some(Student {
                name,
                group,
                scores,
            })
        })
        .collect();

    println!("Parsing questions file...");

    let categories: Vec<Category> = questions
        .records()
        .filter_map(|r| {
            let r = r.ok()?;

            let mut iter = r.iter();
            let description = iter.next()?.to_string();

            let questions = iter.map(str::to_string).collect();

            Some(Category {
                description,
                questions,
            })
        })
        .collect();

    println!("Writing exams...");

    for s in students {
        let mut file = args.output_folder.clone();
        file.push(&s.group.to_string());

        if !file.is_dir() {
            DirBuilder::new().create(&file)?;
        }

        file.push(s.name.to_lowercase().replace(" ", "_"));
        file.set_extension("docx");

        let file = OpenOptions::new().write(true).create(true).open(file)?;

        let mut doc = Docx::new()
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text(s.name.clone())));

        let idx: Vec<usize> = s
            .scores
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, s)| *s < 0.9f32)
            .map(|(i, _)| i)
            .collect();

        for i in idx {
            let category = &categories[i];
            doc = doc.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(category.description.clone())),
            );

            for q in &category.questions {
                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(q.clone())));
            }
        }

        doc.build().pack(file)?;
    }

    Ok(())
}
