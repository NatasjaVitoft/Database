import { useContext } from 'react'
import { Login } from './Login';
import AuthContext from './AuthContext';
import { Projects } from './Projects';
import './App.css'
import { WSProvider } from './WSContextProvider';

function App() {
  const { email, setEmail } = useContext(AuthContext);

  return (
    <>
      {email ? (
        <div>
          <h1>Welcome {email}!</h1>
          <WSProvider>
            <Projects email={email} />
          </WSProvider>
        </div>
      ) :
        <div>
          <Login setEmail={setEmail} />
        </div>
      }
    </>
  )
}

export default App
