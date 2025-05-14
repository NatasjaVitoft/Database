export interface ProjectData {
    name: string,
    format: string,
    owner_email: string,
    id: string,
}

export function getOwnedProjects(email: string): ProjectData[] {

    return [
        {
            name: "evil_plan",
            format: "txt",
            owner_email: email,
            id: "1"
        },
        {
            name: "good_plan",
            format: "txt",
            owner_email: email,
            id: "2",
        },
        {
            name: "plan_b",
            format: "txt",
            owner_email: email,
            id: "3",
        },
    ]
}

export function getCollabProjects(): ProjectData[] {

    return [
        {
            name: "sauce_recipe",
            format: "txt",
            owner_email: "sovs@ost.dk",
            id: "4",
        },
        {
            name: "Fishing_trip",
            format: "txt",
            owner_email: "svensken@lol.dk",
            id: "5",
        },
        {
            name: "blebel",
            format: "txt",
            owner_email: "reje@ost.dk",
            id: "6",
        },
    ]
}

export function getDocumentContent(id: string) {
    switch (id){
        case "1":
            return "step 1: steal underpants, step 2: ????, step 3: PROFIT!!"
        case "2":
            return "Listen to Jesus christ";
        case "3":
            return "ABORT MISSION. Go home instead";
        default:
            return "No content found";
        }
}