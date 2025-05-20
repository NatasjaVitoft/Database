import { useState } from "react";
import { CreateProject } from "./CreateProject";
import { ProjectList } from "./ProjectList";
import { Document } from "./Document";

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
        <Document document={document} setDocument={setDocument} socket={socket} />
      ) : (
        <>
          <h2>Projects blebalaw</h2>
          <ProjectList setDocument={setDocument} email={email} setSocket={setSocket} />
          <CreateProject />
        </>
      )}
    </>
  );
}
