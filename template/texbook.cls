\NeedsTeXFormat{LaTeX2e}

\ProvidesClass{texbook}
\LoadClass[twoside,openright,12pt]{report}
\RequirePackage{amsthm}
\RequirePackage{amssymb}
\RequirePackage{mathrsfs}
\RequirePackage{tikz-cd}

\RequirePackage{amsmath}

\RequirePackage{xr-hyper} 
%hyperlinks and referencing
\RequirePackage[hypertexnames=false]{hyperref} 
\RequirePackage[capitalize,nameinlink,noabbrev]{cleveref}


%setup external file links

\RequirePackage{import}
\ifx\HCode\UnDeFiNeD
    \subimport{../../notes/}{documents.tex}
\fi

\RequirePackage{slashed}

%setup geometry
\RequirePackage[margin=2.5cm]{geometry}
\RequirePackage{graphicx}
\graphicspath{{../../resources}}



\RequirePackage[style=apa]{biblatex}
\addbibresource{../../bibliography.bib}


\hypersetup{
    colorlinks = false,
    citecolor = cyan
}

\usepackage{../../template/style}

\setcounter{secnumdepth}{3}

\theoremstyle{definitionstyle}
\newtheorem{axioma}{Axioma}
\newtheorem{postulado}{Postulado}
\newtheorem{definicion}{Definición}[section]

\theoremstyle{examplestyle}
\newtheorem{ejemplo}{Ejemplo}[section]
\newtheorem{ejemplos}[ejemplo]{Ejemplos}

\theoremstyle{exercisestyle}
\newtheorem{ejercicio}[ejemplo]{Ejercicio}

\theoremstyle{propositionstyle}
\newtheorem{proposicion}{Proposición}[section]
\newtheorem{lema}[proposicion]{Lema}
\newtheorem{teorema}[proposicion]{Teorema}
\newtheorem{corolario}[proposicion]{Corolario}

\theoremstyle{remarkstyle}
\newtheorem{observacion}[definicion]{Observación}

\renewcommand{\maketitle}[4]{
    \onehalfspacing
    \setlength{\abovedisplayskip}{5pt}
    \setlength{\belowdisplayskip}{7pt}

    \begin{titlepage}
    \begin{center}
        \mbox{} \\ [-1.35cm]
        \LARGE
        \HRule{0.06cm} \\ [0.5cm]
        \textbf{\MakeUppercase{\@title}} \HRule{0.06cm}

        \textbf{#1 del Grado en Matemáticas}
        
        \vfill
        {#4}
        \vfill
        % \coffestains
        \normalsize

        \textbf{\@author} \\ \vspace{0.5cm}
        Profesor: #2 \\
        Facultad de Ciencias - Universidad Autónoma de Madrid \\
        #3\\
        \vspace{0.5cm}
        \@date
        \vspace{0.5cm}
    \end{center}
    \end{titlepage}
    \thispagestyle{empty}
    \mbox{}
    \vfill
    \begin{center}
        \textit{Puedes encontrar mi colección completa de apuntes en \href{https://hmarquer.github.io}{https://hmarquer.github.io}.}
        % \textbf{Licencia:} \href{https://creativecommons.org/licenses/by-nc/4.0/deed.es}{Creative Commons Atribución–NoComercial 4.0 Internacional}. Puedes compartir y adaptar el contenido, siempre que cites la autoría y no lo uses con fines comerciales.\\[1em]
        % \href{https://creativecommons.org/licenses/by-nc/4.0/}{
        %     \includesvg[width=3cm]{../../notes/figures/Cc_by-nc_icon.svg}
        % }
    \end{center}
    % \vfill
    % \mbox{}
    \newpage
    \pagenumbering{roman}\tableofcontents
}

\titleformat{\chapter}[block]
  {\normalfont\huge\bfseries}{\thechapter.}{1em}{\Huge}
\titlespacing*{\chapter}{0pt}{-19pt}{0pt}

\usepackage{catchfilebetweentags}

%Fix newlines with catchfilebetweentags
\usepackage{etoolbox}
\makeatletter
\patchcmd{\CatchFBT@Fin@l}{\endlinechar\m@ne}{}
{}{\typeout{Unsuccessful patch!}}
\makeatother

\usepackage{ifthen}

%TeXNotes Commands
\makeatletter
\newcommand{\exhyperref}[3][note]{%
    \@ifundefined{r@#1}%
        {\hyperref[#2-#1]{#3}}% Referencia externa
        {\hyperref[#1]{#3}}% Referencia local (mismo comportamiento)
}

\newcommand{\excref}[2][note]{%
    \@ifundefined{r@#1}%
        {% Referencia externa
            \ifthenelse{\equal{#1}{note}}
                {\hyperref[#2-#1]{\texttt{#2}}}
                {\exhyperref[#1]{#2}{\texttt{#2/}\cref*{#2-#1}}}%
        }%
        {% Referencia local
            \ifthenelse{\equal{#1}{note}}
                {\cref{#1}}
                {\cref{#1}}%
        }%
}

\newcommand{\exref}[2][note]{%
    \@ifundefined{r@#1}%
        {% Referencia externa
            \ifthenelse{\equal{#1}{note}}
                {\hyperref[#2-#1]{\texttt{#2}}}
                {\exhyperref[#1]{#2}{\texttt{#2/}\ref*{#2-#1}}}%
        }%
        {% Referencia local
            \ifthenelse{\equal{#1}{note}}
                {\ref{#1}}
                {\ref{#1}}%
        }%
}
\makeatother

% \newcommand{\excref}[2][note]{\cref{#1}}
% \newcommand{\exref}[2][note]{\ref{#1}}
% \newcommand{\exhyperref}[3][note]{\hyperref[#1]{#3}}

\newcommand{\transclude}[2][note]{\ExecuteMetaData[../../notes/slipbox/#2.tex]{#1}}

% \newcommand{\chapter}[1]{\textbf{\LARGE#1}}

\usepackage{catchfilebetweentags}

%currentdoc command for make4ht fix 
\makeatletter
\newcommand\currentdoc[1]{\edef\@currentlabel{#1}\label{#1}}
\makeatother

\ProcessOptions

