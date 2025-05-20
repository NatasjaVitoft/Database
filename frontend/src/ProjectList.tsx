import type { Dispatch } from 'react';
import { getCollabProjects, getDocumentContent, getOwnedProjects } from './mockdata';
import type { DocumentData } from './Projects';
import { use, useContext } from "react";
import AuthContext from './AuthContext';
import { useEffect, useState } from "react";


interface Project {
    id: string;
    title: string;
    format: string;
    owner_email: string;
}

export interface IProjectListProps {
    setDocument: Dispatch<React.SetStateAction<DocumentData | null>>;
    setSocket: Dispatch<React.SetStateAction<WebSocket | null>>
}

/*
export interface IProjectListProps {
    setDocument: Dispatch<React.SetStateAction<DocumentData | null>>;
    email: string
}
*/

export function ProjectList({ setDocument, email, setSocket }: IProjectListProps) {

    const [ownedProjects, setOwnedProjects] = useState<Project[]>([]);

    const [sharedProjects, setSharedProjects] = useState<Project[]>([]);

    const [error, setError] = useState<string | null>(null);
    

    // Useeffect for owned projects

    useEffect(() => {
        const to_send = { email };

        fetch("http://localhost:3000/get_all_documents_owner", {
            method: "POST",
            headers: {
                "Content-type": "application/json",
                Accept: "application/json",
            },
            body: JSON.stringify(to_send),
        })
            .then((res) => res.json())
            .then((data) => {
                if (data.success) {
                    setOwnedProjects(data.documents);
                } else {
                    console.error("Error from server:", data.message);
                    setError(data.message);
                }
            })
            .catch((err) => {
                console.error("Network error:", err);
            });
    }, [email]);

    // Useeffect for shared projects

    useEffect(() => {
        const to_send = { email };

        fetch("http://localhost:3000/get_all_documents_shared", {
            method: "POST",
            headers: {
                "Content-type": "application/json",
                Accept: "application/json",
            },
            body: JSON.stringify(to_send),
        })
            .then((res) => res.json())
            .then((data) => {
                if (data.success) {
                    setSharedProjects(data.documents);
                } else {
                    console.error("Error from server:", data.message);
                }
            })
            .catch((err) => {
                console.error("Network error:", err);
            });
    }, [email]);


    function handleProjectClick(doc_id: string, name: string, format: string, owner_email: string) {
        // TODO: Implement server fetch
        const socket = new WebSocket(`ws://localhost:3000/ws?user_email=${encodeURIComponent(email)}&document_id=${encodeURIComponent(doc_id)}`);

        // Handle connection open
        socket.onopen = () => {
            console.log("WebSocket connection established");
            setSocket(socket);
        };

        // Handle incoming messages
        socket.onmessage = (event) => {
            console.log("Message from server:", event.data);

            const document: DocumentData = {
                doc_id: doc_id,
                name: name,
                content: event.data,
                format: format,
                owner_email: owner_email,
            }
            setDocument(document);
        };

        // Handle errors
        socket.onerror = (event) => {
            console.error("WebSocket error:", event);
        };

        // Handle connection close
        socket.onclose = () => {
            console.log("WebSocket connection closed");
        };
    }


    return (
        <div className='list-container'>
            <div className='col'>
                <h3>Your projects</h3>
                <p className='error'>{error}</p>
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Format</th>
                            <th>Owner</th>
                        </tr>
                    </thead>
                    <tbody>
                        {ownedProjects.map(p => (
                            <tr key={p.id} onClick={() => handleProjectClick(p.id, p.title, p.format, p.owner_email)}>
                                <td>{p.title}</td>
                                <td>{p.format}</td>
                                <td>{p.owner_email}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
            <div className='col'>
                <h3>Shared projects</h3>
                <table>
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Format</th>
                            <th>Owner</th>
                        </tr>
                    </thead>
                    <tbody>
                        {sharedProjects.map(p => (
                            <tr key={p.id} onClick={() => handleProjectClick(p.id, p.title, p.format, p.owner_email)}>
                                <td>{p.title}</td>
                                <td>{p.format}</td>
                                <td>{p.owner_email}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
