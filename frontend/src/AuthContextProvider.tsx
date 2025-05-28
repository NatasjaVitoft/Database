import { useState } from "react";
import AuthContext from "./AuthContext";

function AuthProvider({ children }: { children: React.ReactNode }) {
    const [email, setEmail] = useState('')

    return ( 
        <AuthContext.Provider value = {{ email, setEmail }}>
            {children}
        </AuthContext.Provider>
     );
}

export default AuthProvider;