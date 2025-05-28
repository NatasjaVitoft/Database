import { createContext } from "react";

export interface IAuth {
    email: string;
    setEmail:  React.Dispatch<React.SetStateAction<string>>;
}

const AuthContext = createContext<IAuth | null>(null);

export default AuthContext;