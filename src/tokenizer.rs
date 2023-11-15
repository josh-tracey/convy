use tokenizers::tokenizer::{Result, Tokenizer};

fn main() -> Result<()> { 
        let tokenizer = Tokenizer::from_pretrained("bert-base-cased", None)?;

        let encoding = tokenizer.encode("Hey there!", false)?;
        println!("{:?}", encoding.get_tokens());
    Ok(())
}
