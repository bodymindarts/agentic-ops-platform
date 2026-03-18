import { useState, useEffect } from "react";
import { fetchCurrentUser, type User } from "./auth";
import { Login } from "./pages/Login";
import { Dashboard } from "./pages/Dashboard";

export function App() {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchCurrentUser()
      .then(setUser)
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="loading">
        <div className="spinner" />
      </div>
    );
  }

  if (!user) {
    return <Login />;
  }

  return <Dashboard user={user} onLogout={() => setUser(null)} />;
}
