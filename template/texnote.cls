\NeedsTeXFormat{LaTeX2e}

\ProvidesClass{texnote}
\LoadClass[12pt]{article}
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
    \subimport{../}{documents.tex}
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

\theoremstyle{definitionstyle}
\newtheorem{axioma}{Axioma}
\newtheorem{postulado}{Postulado}
\newtheorem{definicion}{Definición}

\theoremstyle{examplestyle}
\newtheorem{ejemplo}{Ejemplo}
\newtheorem{ejemplos}[ejemplo]{Ejemplos}

\theoremstyle{exercisestyle}
\newtheorem{ejercicio}[ejemplo]{Ejercicio}

\theoremstyle{propositionstyle}
\newtheorem{proposicion}{Proposición}
\newtheorem{lema}[proposicion]{Lema}
\newtheorem{teorema}[proposicion]{Teorema}
\newtheorem{corolario}[proposicion]{Corolario}

\theoremstyle{remarkstyle}
\newtheorem{observacion}[definicion]{Observación}

\renewcommand{\maketitle}{
    \onehalfspacing
    \setlength{\abovedisplayskip}{5pt}
    \setlength{\belowdisplayskip}{7pt}
    \noindent\@author \hfill \@date
    \begin{center}
        \Huge \textbf{\@title}
    \end{center}
    \noindent
}

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

\newcommand{\transclude}[2][note]{\ExecuteMetaData[#2.tex]{#1}}

\newcommand{\chapter}[1]{\textbf{\LARGE#1}}

\usepackage{catchfilebetweentags}

%currentdoc command for make4ht fix 
\makeatletter
\newcommand\currentdoc[1]{\edef\@currentlabel{#1}\label{#1}}
\makeatother

\ProcessOptions

