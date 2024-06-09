// Handle login form submission as AJAX
document.getElementById('login-form').addEventListener('submit', function(event) {
    event.preventDefault();  // Prevent the form from being submitted normally

    let url = '/login';
    let formData = new FormData(this);
    let data = {};
    data["username"] = formData.get('username');
    data["password"] = formData.get('password');
    let jsonData = JSON.stringify(data)

    console.log("Data: " + data);

    fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: jsonData
    })
    .then(response => {
        if (!response.ok) {
            if (response.status === 401) {
                return response.json().then(data => {
                    throw new Error(data.body.message)
                });
            }
            throw new Error(`HTTP error, status: ${response.status}`);
        } else if (response.redirected) {
            window.location.href = response.url;
        } else {
            throw new Error("Client side error, this application cannot handle server request");
        }
    })
    .catch(error => {
        console.log("error block enetered");
        console.log(`printing ${error.message}`);
        let header = document.getElementById('login-header');
        let div = document.getElementById('error-message');

        if (!div) {
            div = document.createElement('div');
            div.id = 'error-message';
            div.style.color = 'red';
            header.parentNode.insertBefore(div, header.nextSibling);
        }

        div.textContent = error.message;

        setTimeout(() => {
            div.remove();
        }, 2000)
    });
});
