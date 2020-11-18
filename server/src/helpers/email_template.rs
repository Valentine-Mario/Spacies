pub fn verification_email(token: &String) -> String {
    let template = format!(
        "
    <h1>Welcome to Spaces</h1>
    Click the link to verify account<br/>
    <a href='http://localhost:5000/user/vrify?token={}'>click</a>
    ",
        token
    );
    template
}
