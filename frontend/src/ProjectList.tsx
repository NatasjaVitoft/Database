import type { Dispatch } from 'react';
import type { DocumentData, Project } from './Projects';
import { useEffect, useState } from "react";
import { useWebSocket } from './WSContext';


export interface IProjectListProps {
    email: string;
    setDocument: Dispatch<React.SetStateAction<DocumentData | null>>;
    ownedProjects: Project[];
    setOwnedProjects: Dispatch<React.SetStateAction<Project[]>>;
    sharedProjects: Project[];
    setSharedProjects: Dispatch<React.SetStateAction<Project[]>>;
}

export function ProjectList({ setDocument, email, ownedProjects, setOwnedProjects, sharedProjects, setSharedProjects }: IProjectListProps) {
    const { connect } = useWebSocket();

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
    }, [email, ownedProjects, setOwnedProjects, sharedProjects, setSharedProjects]);

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

        const url = `ws://localhost:3000/ws?user_email=${encodeURIComponent(email)}&document_id=${encodeURIComponent(doc_id)}`;

        connect(url, (event) => {
            const document: DocumentData = {
                doc_id,
                name,
                content: event.data,
                format,
                owner_email,
            };
            setDocument(document);
        });
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
