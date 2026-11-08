// Backend commands of this feature, registered in the typed IPC registry
// (`CommandRegistry` in core/ipc). Types only: no runtime code.
import type {
  LoginResponseDto,
  SessionStatusDto,
} from "@/features/auth/data/dto/auth.dto";
import type { UserDto } from "@/features/auth/data/dto/user.dto";

interface CredentialsArgs {
  username: string;
  password: string;
}

declare module "@/core/ipc" {
  interface CommandRegistry {
    account_exists: { args: void; result: boolean };
    register: { args: CredentialsArgs; result: UserDto };
    login: { args: CredentialsArgs; result: LoginResponseDto };
    check_session: { args: { token: string }; result: SessionStatusDto };
    logout: { args: { token: string }; result: void };
    change_password: {
      args: { currentPassword: string; newPassword: string };
      result: void;
    };
  }
}
