
import register from './pages/register.vue'
import Login from "@/Pages/auth/pages/login.vue";


export default [
    {
        path: "/auth/login",
        name: "Login",
        description: "Login page",
        component: Login,
    },

    {
        path: "/auth/register",
        name: "Register",
        description: "Sign up",
        component: register,
    }
]