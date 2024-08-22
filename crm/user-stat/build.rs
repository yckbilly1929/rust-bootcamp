use anyhow::Result;
use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    let builder = tonic_build::configure();
    builder
        .out_dir("src/pb")
        .with_serde(
            &["User, IdContent"],
            true,
            true,
            Some(&[r#"#[serde(rename_all = "camelCase")]"#]),
        )
        .with_derive_builder(
            &[
                "User",
                "IdContent",
                "QueryRequest",
                "RawQueryRequest",
                "TimeQuery",
                "IdQuery",
            ],
            None,
        )
        .with_field_attributes(
            &[
                "User.email",
                "User.name",
                "User.contents",
                "RawQueryRequest.query",
            ],
            &[r#"#[builder(setter(into))]"#],
        )
        .with_field_attributes(
            &["TimeQuery.before", "TimeQuery.after"],
            &[r#"#[builder(setter(into, strip_option))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.timestamps"],
            &[r#"#[builder(setter(each(name="timestamp", into)))]"#],
        )
        .with_field_attributes(
            &["QueryRequest.ids"],
            &[r#"#[builder(setter(each(name="id", into)))]"#],
        )
        .compile(
            &[
                "../protos/user-stats/messages.proto",
                "../protos/user-stats/rpc.proto",
            ],
            &["../protos"],
        )
        .unwrap();
    Ok(())
}
