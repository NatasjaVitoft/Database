import type { Dispatch } from 'react';
import type { DocumentData, Project } from './Projects';
import { useWebSocket } from './WSContext';


export interface IProjectListProps {
    email: string;
    setDocument: Dispatch<React.SetStateAction<DocumentData | null>>;
    ownedProjects: Project[];
    setOwnedProjects: Dispatch<React.SetStateAction<Project[]>>;
    sharedProjects: Project[];
    setSharedProjects: Dispatch<React.SetStateAction<Project[]>>;
}


async function fetchUserRole(doc_id: string, email: string): Promise<string> {
  const res = await fetch("http://localhost:3000/get_user_role", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ document_id: doc_id, email }),
  });
  const data = await res.json();
  if (data.success) {
    return data.user_role[0] || "reader";
  }
  return "reader"; 
}


export function ProjectList({ setDocument, email, ownedProjects, sharedProjects, }: IProjectListProps) {
    const { connect } = useWebSocket();

    async function handleProjectClick(doc_id: string, name: string, format: string, owner_email: string) {
        
        const userRole = await fetchUserRole(doc_id, email);

        if (userRole === "reader") {
            alert("You only have read access to this document.");
        }

        const url = `ws://localhost:3000/ws?user_email=${encodeURIComponent(email)}&document_id=${encodeURIComponent(doc_id)}`;

        connect(url, (event) => {
            const document: DocumentData = {
                doc_id,
                name,
                content: event.data,
                format,
                owner_email,
                userRole,
            };
            setDocument(document);
        });
    }


    return (
        <div className='list-container'>
            <div className='col'>
                <h3>Your projects</h3>
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
