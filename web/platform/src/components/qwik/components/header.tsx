import {
  $,
  type Signal,
  component$,
  useOnDocument,
  useSignal,
  useStylesScoped$,
  useVisibleTask$,
} from "@builder.io/qwik";

import { NavLink } from "./nav-link";

import { GitHub, GitHubIcon, Slack, SlackIcon } from "../../media/icons/icons";
import styles from "./header.css?inline";

const Logo =
  "https://static.wixstatic.com/media/e16379_7fa715a71b41439d93a362b05bbb98a3~mv2.png/v1/fill/w_274,h_52,al_c,lg_1,q_85,enc_auto/e16379_7fa715a71b41439d93a362b05bbb98a3~mv2.png";

const mobileLogo =
  "https://static.wixstatic.com/media/e16379_ebaae466807343b0a880cc8811c53070~mv2.png/v1/fill/w_90,h_80,al_c,lg_1,q_85,enc_auto/Logo.png";

const links = [
  { name: "Home", href: "/" },
  { name: "Product", href: "/product" },
  { name: "Community", href: "/community" },
  { name: "Company", href: "/company" },
  { name: "Resources", href: "/resources" },
  { name: "Docs", href: "/docs/introduction/setup" },
  { name: "Pricing", href: "/pricing" },
];

interface URL {
  pathName: string;
}

const HeaderLogo = component$(() => {
  return (
    <a href="/" class="h-full w-[25vw] z-50 flex justify-center items-center">
      <img
        src={Logo}
        loading="lazy"
        class="w-[179px] hidden md:flex"
        alt="Nativelink Logo"
      />
      <img
        src={mobileLogo}
        loading="lazy"
        class="w-12 md:hidden z-50"
        alt="Nativelink Logo"
      />
    </a>
  );
});

interface DesktopNavProps {
  url: URL;
  scrolled: Signal<boolean>;
}
const DesktopNav = component$<DesktopNavProps>(({ url, scrolled }) => {
  return (
    <nav
      class={`w-[50vw] h-14 hidden md:flex justify-center items-center z-40 transition-all duration-300 ${scrolled.value ? "" : ""}`}
    >
      <ul class="hidden md:flex w-full backdrop-filter backdrop-blur-md text-white px-4 border-white/10 z-60 gap-6 rounded-2xl bg-white/10 border h-12 justify-center items-center">
        {links.map((link) => (
          <NavLink
            key={link.name}
            pathName={url.pathName}
            href={link.href}
            activeClass="font-bold"
          >
            {link.name}
          </NavLink>
        ))}
      </ul>
    </nav>
  );
});

interface MobileNavProps {
  url: URL;
  navState: Signal<boolean>;
}

const MobileNav = component$<MobileNavProps>(({ url, navState }) => {
  return (
    <nav
      class={`fixed top-0 h-full z-40 right-0 w-[100svw] bg-black transition-transform duration-300 ease-in-out ${
        navState.value ? "translate-x-0" : "translate-x-full"
      } md:hidden`}
    >
      <ul class="text-white w-full h-full flex flex-col justify-center items-center gap-10">
        {links.map((link) => (
          <NavLink
            key={link.name}
            pathName={url.pathName}
            href={link.href}
            activeClass="font-bold border rounded-full border-white/20 px-4 py-2"
          >
            {link.name}
          </NavLink>
        ))}
      </ul>
    </nav>
  );
});

interface Hamburger {
  navState: Signal<boolean>;
}

const Hamburger = component$<Hamburger>(({ navState }) => {
  useStylesScoped$(styles);
  return (
    <div class="flex z-60 md:hidden w-[25vw] flex justify-center items-center">
      <button
        onClick$={() => {
          navState.value = !navState.value;
        }}
        class={`hamburger flex justify-center items-center hamburger--slider ${navState.value ? "is-active" : ""}`}
        type="button"
        id="mobile-navigation"
        aria-label="Mobile Navigation"
      >
        <span class="hamburger-box">
          <span class="hamburger-inner" />
        </span>
      </button>
    </div>
  );
});

const Widgets = component$(() => {
  useStylesScoped$(styles);
  return (
    <div class="justify-center w-1/2 md:w-[25vw] flex flex-row items-center gap-2 justify-center text-[16px]">
      <div class="z-60 flex flex-row gap-3 ">
        <a
          target="_blank"
          class="md:hidden"
          href="https://nativelink.slack.com/join/shared_invite/zt-2i2mipfr5-lZAEeWYEy4Eru94b3IOcdg#/shared-invite/email"
          rel="noreferrer"
        >
          <Slack />
        </a>
        <a
          target="_blank"
          class="hidden md:flex hover:rotate-360 transition-all duration-300"
          href="https://nativelink.slack.com/join/shared_invite/zt-2i2mipfr5-lZAEeWYEy4Eru94b3IOcdg#/shared-invite/email"
          rel="noreferrer"
        >
          <SlackIcon />
        </a>
        <a
          class="md:hidden"
          href="https://github.com/tracemachina/nativelink"
          target="_blank"
          rel="noreferrer"
        >
          <GitHub />
        </a>
        <a
          class="hidden md:flex hover:rotate-360 transition-all duration-300"
          href="https://github.com/tracemachina/nativelink"
          target="_blank"
          rel="noreferrer"
        >
          <GitHubIcon />
        </a>
      </div>
      <a
        id="button"
        href="https://app.nativelink.com"
        target="_blank"
        class="hidden md:flex bg-white text-black w-30 hover: p-5 h-8 rounded-xl justify-center items-center"
        rel="noreferrer"
      >
        Try for free
      </a>
    </div>
  );
});

export const Header = component$((url: URL) => {
  const navState = useSignal(false);
  const scrolled = useSignal(false);

  useOnDocument(
    "scrolled",
    $((e: CustomEvent<boolean>) => {
      scrolled.value = e.detail;
    }),
  );

  useVisibleTask$(() => {
    console.info("Welcome to Nativelink");
  });

  return (
    <header
      class={` ${scrolled.value ? "bg-black" : "bg-transparent"}
    fixed  top-0 z-30 flex  h-14 py-10 md:px-0 transition-all duration-500
    w-full justify-between flex-row items-center `}
    >
      <HeaderLogo />
      <DesktopNav url={url} scrolled={scrolled} />
      <Widgets />
      <Hamburger navState={navState} />
      <MobileNav url={url} navState={navState} />
    </header>
  );
});
