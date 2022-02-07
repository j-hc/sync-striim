const URL: &str = "";

#[tokio::main]
async fn main() {
    let mut r = reqwest::Client::new();
    let cl = 2759741;
    let mut t = 0;
    let mut mp3 = Vec::with_capacity(cl);
    while t < cl {
        let end = t + 1024 * 1024;
        println!("{}", format!("bytes={t}-{end}"));
        let resp = r
            .get(URL)
            .header("Range", format!("bytes={t}-{end}"))
            .send()
            .await
            .unwrap();
        let b = resp.bytes().await.unwrap();
        let blen = b.len();
        let w = tokio::io::copy(&mut std::io::Cursor::new(b), &mut mp3)
            .await
            .unwrap();
        println!("b: {blen}, w: {w}");
        t += w as usize;
    }
    println!("{t}");
}
