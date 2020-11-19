pub fn verification_email(token: &String) -> String {
    let template = format!(
        "
    <h1>Welcome to Spaces</h1>
    Click the link to verify account<br/>
    <a href='http://localhost:5000/user/verify?token={}'>click</a>
    ",
        token
    );
    template
}

pub fn forgot_password_email(password: &String) -> String {
    let template = format!(
        "<h1>Password Reset</h1>
        
        Your new password has been set to <b>{}</b>
        Do well to change the password as soon as possible
        ",
        password
    );
    template
}
