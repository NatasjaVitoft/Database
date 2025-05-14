export interface AppendableObject {
    [key: string]: unknown;
}


function fetchData(url: string, callback: (r: unknown) => unknown, method: string, body: object) {


    const headers: AppendableObject = {
      Accept: "application/json",
    };
  
    if (method === "POST" || method === "PUT") {
      headers["Content-Type"] = "application/json";
    }
  
    const options: AppendableObject = makeOptions(method, body);

    if (body) {
      options.body = JSON.stringify(body);
    }
  
    fetch(url, options)
      .then((res) => {
        if (!res.ok) {
          throw new Error(`HTTP error! Status: ${res.status}`);
        }
        return res.json();
      })
      .then((data: Response) => callback(data))
      .catch(handleHttpErrors);
  }

  const makeOptions = (method: string, payload: object) => {
    const opts: AppendableObject = {
      method: method,
      headers: {
        "Content-type": "application/json",
        Accept: "application/json",
      },
    };
  
    if (payload) {
      opts.body = JSON.stringify(payload);
    }
    return opts;
  };

  const handleHttpErrors = (res: Response) => {
    if (!res.ok) {
      return Promise.reject({ status: res.status, fullError: res.json() });
    }
    return res.json();
  }