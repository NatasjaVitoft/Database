import { createContext } from "react";

export interface IAuth {
    isLoggedIn: boolean;
    setIsLoggedIn: React.Dispatch<React.SetStateAction<boolean>>;
    email: string;
    setEmail:  React.Dispatch<React.SetStateAction<string>>;
}

const AuthContext = createContext<IAuth | null>(null);

export default AuthContext;