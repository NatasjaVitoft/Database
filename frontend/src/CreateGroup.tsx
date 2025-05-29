import { useContext, useState, type ChangeEvent, type FormEvent } from 'react';
import AuthContext from './AuthContext';
import { stringToArray } from './api-util';

export interface ICreateGroupProps {
    onCreateGroup: () => void;
}

export default function CreateGroup({ onCreateGroup }: ICreateGroupProps) {

    const { email } = useContext(AuthContext);
    const [msg, setMsg] = useState<string>('');
    const init = {
        name: "",
        members: "",
        role: "editor",
    }

    const [groupInput, setGroupInput] = useState(init);

    function handleInput(e: ChangeEvent<HTMLInputElement | HTMLSelectElement>) {
        setGroupInput({...groupInput, [e.target.id]: e.target.value});
        console.log(groupInput);
    }

    function createGroup(e: FormEvent<HTMLFormElement>) {
        e.preventDefault();
        console.log(`created group`);
        console.log(groupInput);

        const member_arr = stringToArray(groupInput.members);

        const to_send = {
            name: groupInput.name,
            members: member_arr,
            role: groupInput.role,
            owner: email,
        }

        const opts = {
            method: "POST",
            headers: {
                "Content-type": "application/json",
                Accept: "application/json",
            },
            body: JSON.stringify([to_send]),
        };

        fetch("http://localhost:3000/create_group", opts)
            .then((res) => {
                    console.log(res);
                    return res.json();
                })
            .then(data => {
                if (data.success) {
                    onCreateGroup();
                }
                setMsg(data.message);
            })
            .catch((res) => {
                console.log(res);
                setMsg(res.statusText);
        });
    }


    return (
        <div className='login-form'>
            <h3>Create Group</h3>
            <form onSubmit={createGroup}>
                <label>
                    Group Name
                    <input type="text" id="name" onChange={handleInput} />
                </label>
                <label>
                    Members
                    <input type="text" id="members" onChange={handleInput} />
                </label>
                <label>
                    Group Role
                    <select name="role" id="role" onChange={handleInput}>
                        <option value="editor">Editor</option>
                        <option value="reader">Reader</option>
                    </select>
                </label>
                <button type='submit'>Create Group</button>
                <p>{msg}</p>
            </form>
        </div>
    );
}
