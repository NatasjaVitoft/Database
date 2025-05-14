import { useState, type ChangeEvent, type Dispatch } from "react";
import type { DocumentData } from "./Projects";

export interface IDocumentProps {
    document: DocumentData,
    setDocument: Dispatch<React.SetStateAction<DocumentData | null>>;
}


export function Document({ document, setDocument }: IDocumentProps) {

    function onExit() {
        setDocument(null);
    }

    function handleChange(e: ChangeEvent<HTMLTextAreaElement>) {
        setDocument({...document, ['content']: e.target.value})
    }

    return (
        <>
            <h4>Editing:</h4>
            <h5>{document.name}</h5>
            <div className="document-container">
                <textarea name="document-text" id="document-text" value={document.content}  onChange={handleChange}></textarea>
            </div>
            <button onClick={onExit}>Go Back</button>
        </>
    );
}
