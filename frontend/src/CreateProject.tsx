import { useState, type ChangeEvent, type FormEvent } from "react";

export interface ICreateProjectProps { }

export function CreateProject(props: ICreateProjectProps) {
    const init = {
        name: "",
        format: "",
        collab: "",
    };

    const [projectInfo, setProjectInfo] = useState(init);

    function handleInput(e: ChangeEvent<HTMLInputElement>) {
        setProjectInfo({ ...projectInfo, [e.target.id]: e.target.value });
        console.log(projectInfo);
    }

    function onCreateProject(e: FormEvent<HTMLFormElement>) {
        e.preventDefault();
        const collab_arr = projectInfo.collab.split(",");

        const filtered = collab_arr
            .map((s) => {
                return s.trim();
            })
            .filter((s) => {
                return s.length > 1;
            });

        console.log(filtered);

        const to_send = {
            name: projectInfo.name,
            format: projectInfo.format,
            collab: filtered,
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
                    <button type="submit">Create</button>
                </label>
            </form>
        </div>
    );
}
