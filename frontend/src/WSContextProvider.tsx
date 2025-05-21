import React, { useRef, useState, useEffect } from 'react';
import WSContext from './WSContext';

export const WSProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const socketRef = useRef<WebSocket | null>(null);
    const onMessageRef = useRef<(msg: MessageEvent) => void>(() => {});
    const [isConnected, setIsConnected] = useState(false);

    const connect = (url: string, onMessage: (msg: MessageEvent) => void) => {
        if (socketRef.current) {
            socketRef.current.close(); // Close previous
        }

        const socket = new WebSocket(url);
        
        socket.onopen = () => {
            setIsConnected(true);
            console.log("WebSocket connected");
        };

        socket.onmessage = (event) => {
            onMessageRef.current(event);
        };
        
        socket.onerror = (err) => {
            console.error("WebSocket error:", err);
        };
        
        socket.onclose = () => {
            console.log("WebSocket closed");
            setIsConnected(false);
        };

        socketRef.current = socket;
        onMessageRef.current = onMessage;
    };

    const disconnect = () => {
        console.log("Closing: Checking if current")
        if (socketRef.current) {
            console.log("Closing: Current found. Closing")

            setIsConnected(false);
            socketRef.current.close();
            socketRef.current = null;
        }
    };

    const sendMessage = (msg: string) => {
        if (socketRef.current?.readyState === WebSocket.OPEN) {
            socketRef.current.send(msg);
        } else {
            console.warn("WebSocket not open");
        }
    };

    // Clean up on unmount
    useEffect(() => {
        return () => {
            disconnect();
        };
    }, []);

    return (
        <WSContext.Provider value={{ connect, disconnect, sendMessage, isConnected }}>
            {children}
        </WSContext.Provider>
    );
};