import { useAuth } from "@/features/auth/presentation/hooks/use-auth";
import { SessionCountdown } from "@/features/auth/presentation/components/session-countdown";

export function Home() {
  const { user, logout } = useAuth();

  return (
    <main className="home">
      <header className="home-header">
        <SessionCountdown />
        <button onClick={() => void logout()}>Se déconnecter</button>
      </header>

      <section className="home-welcome">
        <h1>Bonjour {user?.username} 👋</h1>
        <p>
          Vous êtes connecté. Vos données de présence sont déverrouillées et
          chiffrées localement.
        </p>
      </section>
    </main>
  );
}
