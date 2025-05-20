import { useState, type ChangeEvent, type Dispatch } from "react";
import type { DocumentData } from "./Projects";

export interface IDocumentProps {
    document: DocumentData,
    setDocument: Dispatch<React.SetStateAction<DocumentData | null>>;
    socket: WebSocket | null,
}


export function Document({ document, setDocument, socket }: IDocumentProps) {


    function onExit() {
        setDocument(null);
        socket?.close()
    }

    function handleChange(e: ChangeEvent<HTMLTextAreaElement>) {
        // setDocument({...document, ['content']: e.target.value})
        socket?.send(e.target.value);
    }

    return (
        <>
            <h2>Editing:</h2>
            <h3>{document.name}</h3>
            <div className="document-container">
                <textarea name="document-text" id="document-text" value={document.content}  onChange={handleChange}></textarea>
            </div>
            <button onClick={onExit}>Go Back</button>
        </>
    );
}
