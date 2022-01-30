export interface Post {
    id: number;
    title: string;
    body: string;
    published: boolean;
    published_date: string;
}

export interface User {
    username: string;
    password: string;
}
