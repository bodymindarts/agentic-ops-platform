export interface User {
  id: string;
  login: string;
  name: string | null;
  avatar_url: string;
}

export async function fetchCurrentUser(): Promise<User | null> {
  try {
    const res = await fetch("/auth/me", { credentials: "include" });
    if (res.ok) {
      return (await res.json()) as User;
    }
    return null;
  } catch {
    return null;
  }
}

export async function logout(): Promise<void> {
  await fetch("/auth/logout", {
    method: "POST",
    credentials: "include",
  });
}
