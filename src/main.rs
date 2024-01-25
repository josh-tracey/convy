use convy::ConventionalCommit;

fn main() {
    let input = r#"feat(deps)!: implement a new feature

    This is a body

    BREAKING-CHANGE: this is a breaking change This is a footer
    "#;

    let commit = ConventionalCommit::parse(input);

    println!("\nConventionalCommit \ncommit_type: {:?} \nscope: {:?} \ndescription: {:?} \nbody: {:?} \nfooter: {:?}\n",
      commit.commit_type,
      commit.scope,
      commit.description,
      commit.body,
      commit.footer
    );

}
