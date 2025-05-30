
import { useState } from "react";
import type { FormEvent, ChangeEvent } from "react";

export interface ILoginProps {
    setEmail: React.Dispatch<React.SetStateAction<string>>
}

export function Login({ setEmail }: ILoginProps)  {
    const init = {
        email: "",
        password: "",
    };
    const [credentials, setCredentials] = useState(init);
    const [msg, setMsg] = useState('');

    function handleInput(e: ChangeEvent<HTMLInputElement>) {
        setCredentials({ ...credentials, [e.target.id]: e.target.value });
        console.log(credentials);
    }

    function performLogin(e: FormEvent<HTMLFormElement>) {
        e.preventDefault()

        const opts = {
            method: 'POST',
            headers: {
                "Content-type": "application/json",
                Accept: "application/json",
            },
            body: JSON.stringify(credentials)
        }

        fetch('http://localhost:3000/login', opts)
            .then(res => {
                if (res.ok) {
                    console.log(res)
                    setEmail(credentials.email)
                }
                else if (res.status == 401) {
                    console.log(res)
                    setMsg("Invalid Credentials")
                }
                else {
                    console.log(res)
                    setMsg(`An error occured with status: ${res.statusText}`)
                }
            }).catch(res => {
                console.log(res)
            })
    }

    return (
        <div className="login-container">
            <form className="login-form" onSubmit={performLogin}>
                <h2>Login</h2>
                <label>
                    E-mail
                    <input
                        type="text"
                        id="email"
                        onChange={handleInput}
                        required
                    />
                </label>
                <label>
                    Password
                    <input
                        type="password"
                        id="password"
                        onChange={handleInput}
                        required
                    />
                </label>
                <button type="submit">Log In</button>
                <p className="error_msg">{msg}</p>
            </form>
        </div>
    );
}