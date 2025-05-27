import { useState } from "react";
import { CreateProject } from "./CreateProject";
import { ProjectList } from "./ProjectList";
import { DocumentEditor } from "./DocumentEditor";
import CreateGroup from "./CreateGroup";

export interface DocumentData {
  doc_id: string;
  name: string;
  content: string;
  format: string;
  owner_email: string;
}

export interface Project {
  id: string;
  title: string;
  format: string;
  owner_email: string;
}

export interface Group {
  id: number;
  name: string;
  role: string;
}

export interface IProjectsProps {
  email: string;
}

export function Projects({ email }: IProjectsProps) {
  const [document, setDocument] = useState<DocumentData | null>(null);
  const [ownedProjects, setOwnedProjects] = useState<Project[]>([]);
  const [sharedProjects, setSharedProjects] = useState<Project[]>([]);
  const [groups, setGroups] = useState<Group[]>([]);

  return (
    <>
      {document ? (
        <DocumentEditor document={document} setDocument={setDocument} />
      ) : (
        <>
          <h2>Projects blebalaw</h2>
          <ProjectList setDocument={setDocument} 
            email={email} ownedProjects={ownedProjects} 
            setOwnedProjects={setOwnedProjects} 
            sharedProjects={sharedProjects} 
            setSharedProjects={setSharedProjects} 
          />
          <div className="list-container">
            <div className="col">
              <CreateProject groups={groups} />
            </div>
            <div className="col">
              <CreateGroup />
            </div>
          </div>
        </>
      )}
    </>
  );
}
