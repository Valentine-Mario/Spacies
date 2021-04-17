use s3::bucket::Bucket;
use s3::creds::Credentials as aws_cred;
use s3::region::Region;
use s3::S3Error;

struct Storage {
    name: String,
    region: Region,
    credentials: aws_cred,
    bucket: String,
}

pub fn aws_func(filename: String, data: Vec<u8>) -> Result<String, S3Error> {
    let aws = Storage {
        name: "aws".into(),
        region: "eu-central-1".parse()?,
        credentials: aws_cred::new(
            Some(&std::env::var("AWS_KEY").expect("AWS KEY not set")),
            Some(&std::env::var("AWS_SECRET").expect("AWS SECRET not set")),
            None,
            None,
            None,
        )?,
        bucket: std::env::var("AWS_BUCKET").expect("AWS SECRET not set"),
    };

    println!("Running {}", aws.name);
    let bucket = Bucket::new(&aws.bucket, aws.region, aws.credentials)?;
    let (_, _code) = bucket.put_object_blocking(&filename, &data)?;
    let file = format!(
        "https://val-mxo.s3.eu-central-1.amazonaws.com/{}",
        &filename
    );

    Ok(file)
}
