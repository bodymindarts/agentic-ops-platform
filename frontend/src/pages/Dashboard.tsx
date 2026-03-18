import { useQuery } from "@apollo/client";
import { HEALTH_QUERY } from "../graphql";
import { logout, type User } from "../auth";

interface Props {
  user: User;
  onLogout: () => void;
}

export function Dashboard({ user, onLogout }: Props) {
  const { data, loading, error } = useQuery<{
    health: { status: string };
  }>(HEALTH_QUERY, { pollInterval: 30000 });

  const handleLogout = async () => {
    await logout();
    onLogout();
  };

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <h1>Agentic Ops</h1>
        <div className="user-info">
          <img
            src={user.avatar_url}
            alt={user.login}
            className="avatar"
          />
          <span className="user-name">{user.name ?? user.login}</span>
          <button onClick={handleLogout} className="logout-button">
            Logout
          </button>
        </div>
      </header>

      <main className="dashboard-content">
        <section className="status-panel">
          <h2>System Status</h2>
          <div className="status-card">
            {loading && <p className="status-loading">Checking...</p>}
            {error && (
              <p className="status-error">
                Error: {error.message}
              </p>
            )}
            {data && (
              <p className="status-ok">
                <span className="status-dot" />
                GraphQL API: {data.health.status}
              </p>
            )}
          </div>
        </section>

        <section className="placeholder-panel">
          <h2>QA Monitoring</h2>
          <div className="placeholder-card">
            <p>Coming soon — automated QA and monitoring dashboards.</p>
          </div>
        </section>

        <section className="placeholder-panel">
          <h2>Agent Activity</h2>
          <div className="placeholder-card">
            <p>Coming soon — real-time agent task tracking.</p>
          </div>
        </section>
      </main>
    </div>
  );
}
