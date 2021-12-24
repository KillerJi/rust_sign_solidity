mod data;
mod entity;
use crate::entity::{claims, prelude::*};
use ooxml::document::SpreadsheetDocument;
use sea_orm::query::*;
use sea_orm::{ActiveModelTrait, Database, DbBackend, EntityTrait, InsertResult, Set};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xlsx = SpreadsheetDocument::open("/root/rust_test_excel.xlsx").unwrap();

    let workbook = xlsx.get_workbook();

    let _sheet_names = workbook.worksheet_names();
    let mut i = 0;
    let mut address: Vec<String> = Vec::new();
    let mut chain_id: Vec<String> = Vec::new();
    let mut token: Vec<String> = Vec::new();
    let mut number: Vec<String> = Vec::new();
    for (sheet_idx, sheet) in workbook.worksheets().iter().enumerate() {
        println!("worksheet {}", sheet_idx);
        println!("worksheet dimension: {:?}", sheet.dimenstion());
        println!("---------DATA---------");
        for rows in sheet.rows() {
            // get cell values
            let cols: Vec<_> = rows
                .into_iter()
                .map(|cell| cell.value().unwrap_or_default())
                .collect();
            address.push(cols[0].clone().to_string());
            chain_id.push(cols[1].clone().to_string());
            token.push(cols[2].clone().to_string());
            number.push(cols[4].clone().to_string());
            i = i + 1;
        }
    }

    let mysql_url = "mysql://root:123456@172.17.0.1:3306/xprotocol?ssl-mode=disabled";
    let pool = Database::connect(mysql_url).await?;
    for j in 0..i - 1 {
        println!("{:?}", j);
        let nonce = data::get_nonce(
            &pool,
            token[j + 1].clone(),
            address[j + 1].clone(),
            chain_id[j + 1].clone(),
        )
        .await;
        println!("nonce {:?}", nonce);
        println!("num {:?}", number[j + 1].clone().parse::<u64>()?);
        // let s = format!(
        //      "INSERT INTO `claims` ( `address`, `chain_id`, `token`, `nonce`, `number` ) VALUES ( {:?}, {:?}, {:?}, {:?}, {:?} );" ,
        //     address[j + 1].clone(),
        //     chain_id[j + 1].clone(),
        //     token[j + 1].clone(),
        //     nonce + 1,
        //     number[j + 1].clone().parse::<u64>()?
        // );
        // println!("{}", s);
        // let _ = pool
        //     .execute(Statement::from_string(DbBackend::MySql, s.to_owned()))
        //     .await?;
        let pear = claims::ActiveModel {
            address: Set(address[j + 1].clone()),
            chain_id: Set(chain_id[j + 1].clone()),
            token: Set(token[j + 1].clone()),
            nonce: Set(nonce + 1),
            number: Set(number[j + 1].clone().parse::<u64>()?),
            ..Default::default()
        };
        let pear = pear.insert(&pool).await?;

        println!("insert success");
    }
    Ok(())
}
