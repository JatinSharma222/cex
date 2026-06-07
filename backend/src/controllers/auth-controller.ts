import bcrypt from "bcryptjs";
import { Request, Response } from "express";
import { prisma } from "../db";
import { authSchema } from "../types/auth_schema";

export async function signup(req:Request, res: Response): Promise<void> {
    const parsedBody = authSchema.safeParse(req.body);
    if(!parsedBody.success) {
        sendValidationError(res, parsedBody.error);
        return;
    }

    const { username, password } = parsedBody.data;

    const hashedPassword = await bcrypt.hash(password, 10);

    try{
        const user = await prisma.user.create({
            data: {
                username,
                password: hashedPassword,
            },
        });

        res.status(201).json({
            token: createToken({userId: user.id}),
            userId: user.id,
            username: user.username,
        });
    } catch (error) {
        res.status(409).json({ error: "username already exists"
        })
    }
}