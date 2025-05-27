import { useState, type ChangeEvent, type Dispatch, type FormEvent } from "react";

import { useContext } from "react";
import AuthContext from "./AuthContext";
import { stringToArray } from "./api-util";
import type { Group } from "./Projects";

export interface ICreateProjectProps {
    groups: Group[]
    setGroups: Dispatch<React.SetStateAction<Group[]>>
 }

export function CreateProject({ groups }) {

    const { email } = useContext(AuthContext);

    const init = {
        name: "",
        format: "",
        collab: "",
        reader: "",
    };

    const [projectInfo, setProjectInfo] = useState(init);

    function handleInput(e: ChangeEvent<HTMLInputElement>) {
        setProjectInfo({ ...projectInfo, [e.target.id]: e.target.value });
        console.log(projectInfo);
    }

    function onCreateProject(e: FormEvent<HTMLFormElement>) {
        
        e.preventDefault();
        const collab_arr = stringToArray(projectInfo.collab)
        const reader_arr = stringToArray(projectInfo.reader)

        const to_send = {
            title: projectInfo.name,
            format: projectInfo.format,
            collaborators: collab_arr,
            readers: reader_arr,
            owner: email, 
        };

        const opts = {
            method: "POST",
            headers: {
                "Content-type": "application/json",
                Accept: "application/json",
            },
            body: JSON.stringify(to_send),
        };

        fetch("http://localhost:3000/save_document_and_relations", opts)
            .then((res) => {
                if (res.ok) {
                    console.log(res);
                } else {
                    console.log(res);
                }
            })
            .catch((res) => {
                console.log(res);
        });
    } 

    return (
        <div className="login-form">
            <h3>Create Project</h3>
            <form onSubmit={onCreateProject}>
                <label>
                    Project name
                    <input type="text" id="name" onChange={handleInput} />
                </label>
                <label>
                    Format
                    <input type="text" id="format" onChange={handleInput} />
                </label>
                <label>
                    Collaborator(s) (E-mail separated with ',')
                    <input type="text" id="collab" onChange={handleInput} />
                </label>
                <label>
                    Reader(s) (E-mail separated with ',')
                    <input type="text" id="reader" onChange={handleInput} />
                </label>
                <button type="submit">Create Project</button>
            </form>
        </div>
    );
}
