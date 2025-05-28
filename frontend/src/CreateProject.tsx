import { useState, type ChangeEvent, type FormEvent } from "react";

import { useContext } from "react";
import AuthContext from "./AuthContext";
import { stringToArray } from "./api-util";
import type { Group } from "./Projects";

export interface ICreateProjectProps {
    groups: Group[]
    onProjectCreated: () => void;
}

export function CreateProject({ groups, onProjectCreated }: ICreateProjectProps) {


    const init = {
        name: "",
        format: "",
        collab: "",
        reader: "",
        groups: [],
    };

    const { email } = useContext(AuthContext);
    const [projectInfo, setProjectInfo] = useState(init);
    const [msg, setMsg] = useState<string>('');


    function handleInput(e: ChangeEvent<HTMLInputElement>) {
        setProjectInfo({ ...projectInfo, [e.target.id]: e.target.value });
        console.log(projectInfo);
    }

    function handleSelectInput(e: ChangeEvent<HTMLSelectElement>) {
        const collection = e.target.selectedOptions;
        const groups = [] as number[];

        for (let i = 0; i < collection.length; i++) {
            groups[i] = parseInt(collection[i].value);
        }

        setProjectInfo({ ...projectInfo, [e.target.id]: groups });
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
            groups: projectInfo.groups
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
                    onProjectCreated();
                    setMsg(res.statusText);
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
                <label>Add collaborator groups
                    <select name="groups" id="groups" onChange={handleSelectInput} multiple>
                        {groups.map(g => (
                            <option key={g.group_id} value={g.group_id}>{g.group_name}</option>
                        ))}
                    </select>
                </label>
                <button type="submit">Create Project</button>
                <p>{msg}</p>
            </form>
        </div>
    );
}
