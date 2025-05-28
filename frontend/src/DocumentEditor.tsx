import { type ChangeEvent, type Dispatch } from "react";
import type { DocumentData } from "./Projects";
import { useWebSocket } from "./WSContext";

export interface IDocumentEditorProps {
    document: DocumentData,
    setDocument: Dispatch<React.SetStateAction<DocumentData | null>>;
}


export function DocumentEditor({ document, setDocument}: IDocumentEditorProps) {
    const { sendMessage, disconnect } = useWebSocket();

    function onExit() {
        disconnect();
        setDocument(null);
    }

    function handleChange(e: ChangeEvent<HTMLTextAreaElement>) {
        // setDocument({...document, ['content']: e.target.value})
        sendMessage(e.target.value);
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
