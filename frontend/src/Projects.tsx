import { useState } from "react";
import { CreateProject } from "./CreateProject";
import { ProjectList } from "./ProjectList";
import {  DocumentEditor } from "./DocumentEditor";

export interface DocumentData {
  doc_id: string;
  name: string;
  content: string;
  format: string;
  owner_email: string;
}

export interface IProjectsProps {
  email: string;
}

export function Projects({ email }: IProjectsProps) {
  const [document, setDocument] = useState<DocumentData | null>(null);
  const [socket, setSocket] = useState<WebSocket | null>(null);

  return (
    <>
      {document ? (
        <DocumentEditor document={document} setDocument={setDocument} />
      ) : (
        <>
          <h2>Projects blebalaw</h2>
          <ProjectList setDocument={setDocument} email={email} />
          <CreateProject />
        </>
      )}
    </>
  );
}
