import { createContext, useContext } from 'react';

type WebSocketContextType = {
    connect: (url: string, onMessage: (msg: MessageEvent) => void) => void;
    disconnect: () => void;
    sendMessage: (msg: string) => void;
    isConnected: boolean;
};

export const useWebSocket = (): WebSocketContextType => {
    const ctx = useContext(WebSocketContext);
    if (!ctx) throw new Error("useWebSocket must be used within WebSocketProvider");
    return ctx;
};

const WebSocketContext = createContext<WebSocketContextType | undefined>(undefined);

export default WebSocketContext