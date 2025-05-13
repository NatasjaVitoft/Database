import { useState, useContext } from 'react'
import { Document } from './Document'
import { Login } from './Login';
import AuthContext from './AuthContext';
import { Projects } from './Projects';
import './App.css'

function App() {
  const { isLoggedIn, setIsLoggedIn, email, setEmail } = useContext(AuthContext);

  return (
    <>
      {isLoggedIn ? (
        <div>
          <Projects></Projects>
        </div>
      ) :
        <div>
          <Login setEmail={setEmail} setIsLoggedIn={setIsLoggedIn}/>
        </div>
      }
    </>
  )
}

export default App
