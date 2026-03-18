import { useState, useEffect } from "react";
import { gql, useQuery } from "@apollo/client";

const HEALTH_QUERY = gql`
  query Health {
    health {
      status
    }
  }
`;

interface User {
  login: string;
  avatar_url: string;
  name: string | null;
}

function LoginPage() {
  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <h1 style={styles.title}>Agentic Ops</h1>
        <p style={styles.subtitle}>Operations platform for agentic workflows</p>
        <a href="/auth/login" style={styles.button}>
          Sign in with GitHub
        </a>
      </div>
    </div>
  );
}

function Dashboard({ user }: { user: User }) {
  const { data, loading, error } = useQuery(HEALTH_QUERY, {
    pollInterval: 30000,
  });

  const handleLogout = async () => {
    await fetch("/auth/logout", { method: "POST", credentials: "include" });
    window.location.href = "/";
  };

  return (
    <div style={styles.dashboardContainer}>
      <header style={styles.header}>
        <h2 style={styles.headerTitle}>Agentic Ops</h2>
        <div style={styles.headerRight}>
          <img
            src={user.avatar_url}
            alt={user.login}
            style={styles.avatar}
          />
          <span style={styles.userName}>{user.name || user.login}</span>
          <button onClick={handleLogout} style={styles.logoutButton}>
            Logout
          </button>
        </div>
      </header>

      <main style={styles.main}>
        <div style={styles.statusCard}>
          <h3 style={styles.cardTitle}>System Health</h3>
          {loading && <p style={styles.statusText}>Checking...</p>}
          {error && (
            <p style={{ ...styles.statusText, color: "#e53e3e" }}>
              Error: {error.message}
            </p>
          )}
          {data && (
            <p style={styles.statusText}>
              <span style={styles.statusDot}>●</span>{" "}
              {data.health.status === "ok" ? "All systems operational" : data.health.status}
            </p>
          )}
        </div>

        <div style={styles.placeholderCard}>
          <h3 style={styles.cardTitle}>Monitoring</h3>
          <p style={styles.placeholderText}>
            Agent monitoring and workflow dashboards coming soon.
          </p>
        </div>
      </main>
    </div>
  );
}

export default function App() {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch("/auth/me", { credentials: "include" })
      .then((res) => {
        if (res.ok) return res.json();
        throw new Error("Not authenticated");
      })
      .then((data: User) => setUser(data))
      .catch(() => setUser(null))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div style={styles.container}>
        <p style={styles.loadingText}>Loading...</p>
      </div>
    );
  }

  return user ? <Dashboard user={user} /> : <LoginPage />;
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    minHeight: "100vh",
    backgroundColor: "#0f1117",
    fontFamily:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  },
  card: {
    backgroundColor: "#1a1d27",
    borderRadius: "12px",
    padding: "48px",
    textAlign: "center" as const,
    border: "1px solid #2d3148",
    maxWidth: "400px",
    width: "100%",
    margin: "0 16px",
  },
  title: {
    color: "#e2e8f0",
    fontSize: "28px",
    fontWeight: 700,
    margin: "0 0 8px",
  },
  subtitle: {
    color: "#8b92a8",
    fontSize: "14px",
    margin: "0 0 32px",
  },
  button: {
    display: "inline-block",
    backgroundColor: "#2d3748",
    color: "#e2e8f0",
    padding: "12px 24px",
    borderRadius: "8px",
    textDecoration: "none",
    fontSize: "15px",
    fontWeight: 500,
    border: "1px solid #4a5568",
    transition: "background-color 0.2s",
  },
  dashboardContainer: {
    minHeight: "100vh",
    backgroundColor: "#0f1117",
    fontFamily:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  },
  header: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "16px 24px",
    backgroundColor: "#1a1d27",
    borderBottom: "1px solid #2d3148",
  },
  headerTitle: {
    color: "#e2e8f0",
    fontSize: "18px",
    fontWeight: 600,
    margin: 0,
  },
  headerRight: {
    display: "flex",
    alignItems: "center",
    gap: "12px",
  },
  avatar: {
    width: "32px",
    height: "32px",
    borderRadius: "50%",
  },
  userName: {
    color: "#cbd5e0",
    fontSize: "14px",
  },
  logoutButton: {
    backgroundColor: "transparent",
    color: "#8b92a8",
    border: "1px solid #2d3148",
    padding: "6px 12px",
    borderRadius: "6px",
    cursor: "pointer",
    fontSize: "13px",
  },
  main: {
    padding: "24px",
    maxWidth: "800px",
    margin: "0 auto",
    display: "flex",
    flexDirection: "column" as const,
    gap: "16px",
  },
  statusCard: {
    backgroundColor: "#1a1d27",
    borderRadius: "8px",
    padding: "24px",
    border: "1px solid #2d3148",
  },
  placeholderCard: {
    backgroundColor: "#1a1d27",
    borderRadius: "8px",
    padding: "24px",
    border: "1px solid #2d3148",
  },
  cardTitle: {
    color: "#e2e8f0",
    fontSize: "16px",
    fontWeight: 600,
    margin: "0 0 12px",
  },
  statusText: {
    color: "#68d391",
    fontSize: "14px",
    margin: 0,
    display: "flex",
    alignItems: "center",
    gap: "8px",
  },
  statusDot: {
    fontSize: "12px",
  },
  placeholderText: {
    color: "#8b92a8",
    fontSize: "14px",
    margin: 0,
  },
  loadingText: {
    color: "#8b92a8",
    fontSize: "16px",
  },
};
