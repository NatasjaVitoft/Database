import { useState } from "react";
import AuthContext from "./AuthContext";

function AuthProvider({ children }: { children: React.ReactNode }) {
    const [isLoggedIn, setIsLoggedIn] = useState(false)
    const [email, setEmail] = useState('')

    return ( 
        <AuthContext.Provider value = {{ isLoggedIn, setIsLoggedIn, email, setEmail }}>
            {children}
        </AuthContext.Provider>
     );
}

export default AuthProvider;