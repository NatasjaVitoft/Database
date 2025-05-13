
import { useState } from "react";
import type { FormEvent, ChangeEvent } from "react";

export function Login({ setEmail, setIsLoggedIn }) {
    const init = {
        email: "",
        password: "",
      };
      const [credentials, setCredentials] = useState(init);

    function handleInput(e: ChangeEvent<HTMLInputElement>) {
        setCredentials({ ...credentials, [e.target.id]: e.target.value });
        console.log(credentials);
    }

    function performLogin(e: FormEvent<HTMLFormElement>) {
        e.preventDefault()
        //  TODO: Perform login on server.
        //  If succesful, add email to authcontext
        // if not display error
        setEmail(credentials.email);
        setIsLoggedIn(true);
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
            </form>
        </div>
    );
}