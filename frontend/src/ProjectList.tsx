import type { Dispatch } from 'react';
import { getCollabProjects, getDocumentContent, getOwnedProjects } from './mockdata';
import type { DocumentData } from './Projects';

export interface IProjectListProps {
    setDocument: Dispatch<React.SetStateAction<DocumentData | null>>;
    email: string
}

export function ProjectList({ setDocument, email }: IProjectListProps) {
    const ownedProjects = getOwnedProjects(email);
    const sharedProjects = getCollabProjects();

    function handleProjectClick(doc_id: string, name: string, format: string, owner_email: string) {
        // TODO: Implement server fetch
        const cont = getDocumentContent(doc_id);

        setDocument({ doc_id: doc_id, name: name, content: cont, format: format, owner_email: owner_email })
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
                            <tr key={p.id} onClick={() => handleProjectClick(p.id, p.name, p.format, p.owner_email)}>
                                <td>{p.name}</td>
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
                            <tr key={p.id}>
                                <td>{p.name}</td>
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
