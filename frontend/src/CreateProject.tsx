import { useState, type ChangeEvent, type FormEvent } from "react";

export interface ICreateProjectProps { }

export function CreateProject(props: ICreateProjectProps) {
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

    function stringToArray(str: string): string[] {
        if (str) {
            return str
                .split(",")
                .map((s) => {
                    return s.trim();
                })
                .filter((s) => {
                    return s.length > 1;
                });
        }
        return [];
    }

    function onCreateProject(e: FormEvent<HTMLFormElement>) {
        e.preventDefault();
        const collab_arr = stringToArray(projectInfo.collab)
        const reader_arr = stringToArray(projectInfo.reader)

        const to_send = {
            name: projectInfo.name,
            format: projectInfo.format,
            collab: collab_arr,
            readers: reader_arr,
        };

        // TODO: submit to backend
        console.log(to_send);
    }

    return (
        <div className="login-form">
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
                <button type="submit">Create</button>
            </form>
        </div>
    );
}
