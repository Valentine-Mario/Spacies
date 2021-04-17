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

pub fn invite_user(name: &String, token: &String) -> String {
    let template = format!(
        "
    <h1>You have been invited to {} Space</h1>
    Follow the link to complete registration<br/>
    <a href='http://localhost:5000/spacies/invitepage?token={}'>click</a>
    ",
        name, token
    );
    template
}

pub fn added_user(name: &String) -> String {
    let template = format!(
        "
    <h1>You have been added to {} space</h1>
    ",
        name
    );
    template
}

pub fn notify_folder(title: &String, body: &String) -> String {
    let template = format!(
        "
        <h3>Notification to {}</h3>
        <p>{}</p>
    ",
        title, body
    );
    template
}

pub fn send_reminder(title: &String, name: &String, body: &String) -> String {
    let template = format!(
        "
        <h3>Reminder for {}</h3>

        <p>This is to remind membrs of {} Space of the follwing event today:</p>
       <p> {}</p>
        ",
        title, name, body
    );
    template
}
