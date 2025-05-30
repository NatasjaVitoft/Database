import { useContext, useEffect, useState } from "react";
import { CreateProject } from "./CreateProject";
import { ProjectList } from "./ProjectList";
import { DocumentEditor } from "./DocumentEditor";
import CreateGroup from "./CreateGroup";
import AuthContext from "./AuthContext";

export interface DocumentData {
  doc_id: string;
  name: string;
  content: string;
  format: string;
  owner_email: string;
  userRole?: string;
}

export interface Project {
  id: string;
  title: string;
  format: string;
  owner_email: string;
}

export interface Group {
  group_name: string;
  owner_email: string;
  group_role: string;
  group_id: number;
}

export function Projects() {
  const { email } = useContext(AuthContext);
  const [document, setDocument] = useState<DocumentData | null>(null);
  const [ownedProjects, setOwnedProjects] = useState<Project[]>([]);
  const [sharedProjects, setSharedProjects] = useState<Project[]>([]);
  const [groups, setGroups] = useState<Group[]>([]);

  function fetchOwnedProjects() {
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
        }
      })
      .catch((err) => {
        console.error("Network error:", err);
      });
  }

  function fetchSharedProjects() {
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
  }

  function fetchGroups() {
    const to_send = { email };

    fetch("http://localhost:3000/get_groups_by_owner", {
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
                setGroups(data.groups);
                console.log("received:")
                console.log(data.groups);
                console.log("after being set:");
                console.log(groups);
            } else {
                console.error("Error from server:", data.message);
            }
        })
        .catch((err) => {
            console.error("Network error:", err);
        });
  }

  // Fetch projects on render
  useEffect(() => {
    fetchOwnedProjects();
    fetchSharedProjects();
    fetchGroups();
  }, []);

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
              <CreateProject groups={groups}
                onProjectCreated={() => {
                  fetchOwnedProjects();
                  fetchSharedProjects();
                }} />

            </div>
            <div className="col">
              <CreateGroup onCreateGroup={() => {
                fetchGroups();
              }}/>
            </div>
          </div>
        </>
      )}
    </>
  );
}
