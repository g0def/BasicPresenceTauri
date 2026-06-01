import type { UserDto } from "@/features/auth/data/dto/user.dto";
import type { User } from "@/features/auth/domain/entities/user";

export function toUser(dto: UserDto): User {
  return {
    id: dto.id,
    username: dto.username,
    createdAt: dto.createdAt,
    updatedAt: dto.updatedAt,
  };
}
